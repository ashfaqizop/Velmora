-- Velmora Theatre: scheduled screenings everyone watches at the same moment,
-- with a live chat that is never saved. Spec: docs/theatre.md.
--
-- Run in Supabase's SQL Editor AFTER schema.sql, sync.sql, social.sql and
-- activity.sql. Safe to run again. Needs the pg_cron extension (Database >
-- Extensions > pg_cron) for the minute-by-minute job at the end; without it
-- everything else still works and the job is skipped with a notice.
--
-- The lineup is written by server code (site/api/theatre) with the service
-- key. People book, cancel, enter and chat only through the functions below.

-- ── Velmora Max, as the database sees it ───────────────────────────────────
-- Max lives in the license server (Redis); the site mirrors who has it here so
-- booking rules can check it. Written only with the service key.

create table if not exists public.max_accounts (
  user_id uuid primary key references auth.users (id) on delete cascade,
  since timestamptz not null default now()
);
alter table public.max_accounts enable row level security;

create or replace function public.is_max(uid uuid)
returns boolean
language sql
stable
security definer
set search_path = ''
as $$ select exists (select 1 from public.max_accounts m where m.user_id = uid) $$;

-- ── Screenings ─────────────────────────────────────────────────────────────

create table if not exists public.theatre_screenings (
  id uuid primary key default gen_random_uuid(),
  -- The UTC day the lineup belongs to.
  day date not null,
  k text not null check (k ~ '^movie/[0-9]{1,9}$'),
  name text not null check (char_length(name) between 1 and 200),
  year integer,
  poster text,
  backdrop text,
  runtime integer not null check (runtime between 20 and 600),
  kind text not null default 'trending' check (kind in ('trending', 'pick')),
  starts_at timestamptz not null,
  ends_at timestamptz not null,
  -- draft: waiting on source checks; scheduled: on the lineup; cancelled: dropped.
  status text not null default 'draft' check (status in ('draft', 'scheduled', 'cancelled')),
  -- open: drop in without a ticket; fixed: ticket required, can sell out.
  mode text not null default 'open' check (mode in ('open', 'fixed')),
  capacity integer not null default 60 check (capacity between 1 and 1000),
  expected integer not null default 0,
  booked integer not null default 0,
  waitlisted integer not null default 0,
  -- Proven: Velmora users already played it, so sources exist.
  proven boolean not null default false,
  -- Booking opens for Max first, then everyone.
  max_opens_at timestamptz,
  opens_at timestamptz,
  -- Set by the minute job once no-shows were released / the screening wrapped up.
  released boolean not null default false,
  settled boolean not null default false,
  created_at timestamptz not null default now(),
  constraint theatre_times check (ends_at > starts_at)
);
create index if not exists theatre_screenings_day on public.theatre_screenings (day, starts_at);
create index if not exists theatre_screenings_start on public.theatre_screenings (starts_at);
create index if not exists theatre_screenings_k on public.theatre_screenings (k, starts_at desc);

alter table public.theatre_screenings enable row level security;
drop policy if exists "lineup is public" on public.theatre_screenings;
create policy "lineup is public" on public.theatre_screenings for select to anon, authenticated
  using (status in ('scheduled', 'cancelled'));

-- ── Your hand on the lineup (edit in Supabase's Table Editor) ──────────────
--   pin:   put this title on `day` (UTC) if it fits
--   block: never schedule this title (day ignored)

create table if not exists public.theatre_overrides (
  id bigint generated always as identity primary key,
  action text not null check (action in ('pin', 'block')),
  k text not null check (k ~ '^movie/[0-9]{1,9}$'),
  day date,
  note text not null default '',
  created_at timestamptz not null default now()
);
alter table public.theatre_overrides enable row level security;

-- ── Tickets ────────────────────────────────────────────────────────────────

create table if not exists public.theatre_tickets (
  screening_id uuid not null references public.theatre_screenings (id) on delete cascade,
  user_id uuid not null references public.profiles (id) on delete cascade,
  -- booked, waitlist, cancelled; released (no-show seat given away);
  -- attended / noshow once the screening is over.
  status text not null check (status in ('booked', 'waitlist', 'cancelled', 'released', 'attended', 'noshow')),
  is_max boolean not null default false,
  created_at timestamptz not null default now(),
  joined_at timestamptz,
  primary key (screening_id, user_id)
);
create index if not exists theatre_tickets_user on public.theatre_tickets (user_id, created_at desc);
create index if not exists theatre_tickets_wait on public.theatre_tickets (screening_id, status, is_max desc, created_at);

alter table public.theatre_tickets enable row level security;
-- Your own tickets; other people's only while they share what they watch
-- (for "people you follow are going" and attendance stamps).
drop policy if exists "tickets you may see" on public.theatre_tickets;
create policy "tickets you may see" on public.theatre_tickets for select to anon, authenticated
  using (user_id = (select auth.uid()) or (status in ('booked', 'attended') and public.profile_sharing(user_id)));

-- ── Time ───────────────────────────────────────────────────────────────────

-- The server's clock, so every app agrees on where the movie is.
create or replace function public.theatre_now()
returns timestamptz
language sql
volatile
as $$ select clock_timestamp() $$;
grant execute on function public.theatre_now() to anon, authenticated;

-- Chat and the lobby open 30 minutes before and close 30 minutes after.
create or replace function public.theatre_window_open(s public.theatre_screenings)
returns boolean
language sql
stable
as $$ select s.status = 'scheduled' and now() between s.starts_at - interval '30 minutes' and s.ends_at + interval '30 minutes' $$;

-- ── Booking ────────────────────────────────────────────────────────────────

-- When booking opens for this person: Max first, everyone 6 hours later, and
-- an hour later still after 3 no-shows in 30 days.
create or replace function public.theatre_opens_for(s public.theatre_screenings, uid uuid)
returns timestamptz
language sql
stable
security definer
set search_path = ''
as $$
  select (case when public.is_max(uid) then coalesce(s.max_opens_at, s.opens_at) else s.opens_at end)
    + case when (select count(*) from public.theatre_tickets t
                 where t.user_id = uid and t.status = 'noshow' and t.created_at > now() - interval '30 days') >= 3
           then interval '1 hour' else interval '0' end
$$;

-- Move people up from the waitlist into free seats (Max first, then in order).
create or replace function public.theatre_promote(sid uuid)
returns integer
language plpgsql
security definer
set search_path = ''
as $$
declare
  s public.theatre_screenings;
  free integer;
  moved integer := 0;
begin
  select * into s from public.theatre_screenings where id = sid for update;
  if not found or s.mode <> 'fixed' then return 0; end if;
  free := s.capacity - s.booked;
  if free <= 0 then return 0; end if;
  with up as (
    select t.user_id from public.theatre_tickets t
    where t.screening_id = sid and t.status = 'waitlist'
    order by t.is_max desc, t.created_at
    limit free
  )
  update public.theatre_tickets t set status = 'booked'
  from up where t.screening_id = sid and t.user_id = up.user_id;
  get diagnostics moved = row_count;
  if moved > 0 then
    update public.theatre_screenings
    set booked = booked + moved, waitlisted = greatest(waitlisted - moved, 0)
    where id = sid;
  end if;
  return moved;
end;
$$;
revoke all on function public.theatre_promote(uuid) from public, anon, authenticated;

-- Book a seat. Returns 'booked', 'waitlist' or 'already'.
create or replace function public.theatre_book(sid uuid)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  s public.theatre_screenings;
  current text;
  result text;
begin
  if me is null then raise exception 'sign_in' using errcode = 'P0001'; end if;
  if not public.profile_visible(me) then raise exception 'not_allowed' using errcode = 'P0001'; end if;
  select * into s from public.theatre_screenings where id = sid for update;
  if not found or s.status <> 'scheduled' then raise exception 'not_bookable' using errcode = 'P0001'; end if;
  if now() >= s.ends_at then raise exception 'over' using errcode = 'P0001'; end if;
  if s.opens_at is not null and now() < public.theatre_opens_for(s, me) then
    raise exception 'not_open_yet' using errcode = 'P0001';
  end if;

  select status into current from public.theatre_tickets where screening_id = sid and user_id = me;
  if current in ('booked', 'waitlist', 'attended') then return 'already'; end if;

  -- After no-shows were released, fixed seats only come from the waitlist.
  result := case when s.mode = 'fixed' and (s.booked >= s.capacity or s.released) then 'waitlist' else 'booked' end;
  insert into public.theatre_tickets (screening_id, user_id, status, is_max, created_at)
  values (sid, me, result, public.is_max(me), now())
  on conflict (screening_id, user_id) do update set status = excluded.status, is_max = excluded.is_max, created_at = now(), joined_at = null;

  if result = 'booked' then
    update public.theatre_screenings set booked = booked + 1 where id = sid;
  else
    update public.theatre_screenings set waitlisted = waitlisted + 1 where id = sid;
  end if;
  return result;
end;
$$;
revoke all on function public.theatre_book(uuid) from public, anon;
grant execute on function public.theatre_book(uuid) to authenticated;

-- Cancel. A freed fixed seat goes to the waitlist at once.
create or replace function public.theatre_cancel(sid uuid)
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  was text;
begin
  if me is null then return; end if;
  perform 1 from public.theatre_screenings where id = sid for update;
  select status into was from public.theatre_tickets where screening_id = sid and user_id = me;
  if was not in ('booked', 'waitlist') then return; end if;
  update public.theatre_tickets set status = 'cancelled' where screening_id = sid and user_id = me;
  if was = 'booked' then
    update public.theatre_screenings set booked = greatest(booked - 1, 0) where id = sid;
    perform public.theatre_promote(sid);
  else
    update public.theatre_screenings set waitlisted = greatest(waitlisted - 1, 0) where id = sid;
  end if;
end;
$$;
revoke all on function public.theatre_cancel(uuid) from public, anon;
grant execute on function public.theatre_cancel(uuid) to authenticated;

-- ── Entering a screening ───────────────────────────────────────────────────

-- May this person be in the screening (lobby, show, after-chat)?
create or replace function public.theatre_can_enter(sid uuid, uid uuid)
returns boolean
language sql
stable
security definer
set search_path = ''
as $$
  select exists (
    select 1 from public.theatre_screenings s
    where s.id = sid
      and public.theatre_window_open(s)
      and public.profile_visible(uid)
      and (s.mode = 'open'
           or exists (select 1 from public.theatre_tickets t
                      where t.screening_id = s.id and t.user_id = uid and t.status in ('booked', 'attended')))
  )
$$;

-- Come into the lobby or the show. Records that you came (so your seat isn't
-- given away), and returns the screening plus the server clock.
create or replace function public.theatre_enter(sid uuid)
returns jsonb
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  s public.theatre_screenings;
  t public.theatre_tickets;
begin
  if me is null then raise exception 'sign_in' using errcode = 'P0001'; end if;
  select * into s from public.theatre_screenings where id = sid;
  if not found or s.status <> 'scheduled' then raise exception 'not_found' using errcode = 'P0001'; end if;
  if not public.theatre_window_open(s) then
    return jsonb_build_object('allowed', false, 'reason', case when now() < s.starts_at then 'not_yet' else 'over' end,
                              'server_now', clock_timestamp());
  end if;

  select * into t from public.theatre_tickets where screening_id = sid and user_id = me;
  if s.mode = 'open' and (t.user_id is null or t.status in ('cancelled', 'released')) then
    -- Walk-in: an open screening needs no ticket; this one records attendance.
    insert into public.theatre_tickets (screening_id, user_id, status, is_max, joined_at)
    values (sid, me, 'booked', public.is_max(me), clock_timestamp())
    on conflict (screening_id, user_id) do update set status = 'booked', joined_at = coalesce(public.theatre_tickets.joined_at, clock_timestamp());
    update public.theatre_screenings set booked = booked + 1 where id = sid;
  elsif t.status in ('booked', 'attended') then
    update public.theatre_tickets set joined_at = coalesce(joined_at, clock_timestamp()) where screening_id = sid and user_id = me;
  else
    return jsonb_build_object('allowed', false, 'reason', case when t.status = 'waitlist' then 'waitlist' else 'no_ticket' end,
                              'server_now', clock_timestamp());
  end if;

  return jsonb_build_object(
    'allowed', true,
    'server_now', clock_timestamp(),
    'id', s.id, 'k', s.k, 'name', s.name, 'runtime', s.runtime,
    'starts_at', s.starts_at, 'ends_at', s.ends_at,
    'mode', s.mode, 'capacity', s.capacity
  );
end;
$$;
revoke all on function public.theatre_enter(uuid) from public, anon;
grant execute on function public.theatre_enter(uuid) to authenticated;

-- ── Live chat (never stored) ───────────────────────────────────────────────
-- Messages go out through Supabase Realtime to the private topic
-- "theatre-chat:<id>", which people can only listen to. Only this function
-- sends there, after its checks. What is kept: when each person last spoke
-- (and a hash of their last message, to stop repeats), and a per-minute count
-- for slow mode. Never the text.

create table if not exists public.theatre_chat_state (
  screening_id uuid not null references public.theatre_screenings (id) on delete cascade,
  user_id uuid not null references public.profiles (id) on delete cascade,
  last_at timestamptz not null,
  last_hash text not null,
  primary key (screening_id, user_id)
);
alter table public.theatre_chat_state enable row level security;

create table if not exists public.theatre_chat_rate (
  screening_id uuid not null references public.theatre_screenings (id) on delete cascade,
  minute timestamptz not null,
  n integer not null default 0,
  primary key (screening_id, minute)
);
alter table public.theatre_chat_rate enable row level security;

create or replace function public.theatre_say(sid uuid, body text, movie_ms integer default null)
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  p public.profiles;
  msg text := btrim(regexp_replace(coalesce(body, ''), '\s+', ' ', 'g'));
  busy integer;
  gap interval;
  st public.theatre_chat_state;
  h text;
  w text;
begin
  if me is null then raise exception 'sign_in' using errcode = 'P0001'; end if;
  if not public.theatre_can_enter(sid, me) then raise exception 'not_in_screening' using errcode = 'P0001'; end if;
  if char_length(msg) not between 1 and 200 then raise exception 'chat_length' using errcode = 'P0001'; end if;
  if msg ~* '(https?://|www\.|[a-z0-9-]+\.(com|net|org|io|gg|xyz|ru|me|tv|co|ly|link)\b)' then
    raise exception 'chat_link' using errcode = 'P0001';
  end if;
  for w in select b.word from public.blocked_words b loop
    if msg ~* ('(^|[^a-z0-9])' || regexp_replace(w, '([.*+?^${}()|\[\]\\])', '\\\1', 'g') || '($|[^a-z0-9])') then
      raise exception 'chat_blocked' using errcode = 'P0001';
    end if;
  end loop;

  -- Slow mode: the busier the room, the longer between your messages.
  select coalesce(sum(r.n), 0) into busy from public.theatre_chat_rate r
  where r.screening_id = sid and r.minute > now() - interval '1 minute';
  gap := case when busy > 300 then interval '10 seconds' when busy > 120 then interval '5 seconds'
              when busy > 40 then interval '2.5 seconds' else interval '1 second' end;
  h := md5(lower(msg));
  select * into st from public.theatre_chat_state where screening_id = sid and user_id = me;
  if found then
    if st.last_at > now() - gap then raise exception 'chat_slow' using errcode = 'P0001', detail = ceil(extract(epoch from (st.last_at + gap - now())))::text; end if;
    if st.last_hash = h and st.last_at > now() - interval '60 seconds' then raise exception 'chat_repeat' using errcode = 'P0001'; end if;
  end if;
  insert into public.theatre_chat_state (screening_id, user_id, last_at, last_hash) values (sid, me, now(), h)
    on conflict (screening_id, user_id) do update set last_at = now(), last_hash = h;
  insert into public.theatre_chat_rate (screening_id, minute, n) values (sid, date_trunc('minute', now()), 1)
    on conflict (screening_id, minute) do update set n = public.theatre_chat_rate.n + 1;

  select * into p from public.profiles where id = me;
  perform realtime.send(
    jsonb_build_object(
      'id', gen_random_uuid(),
      'uid', p.id, 'username', p.username, 'display_name', p.display_name, 'avatar', p.avatar, 'verified', p.verified,
      'body', msg,
      'at', movie_ms,
      'ts', now()
    ),
    'chat',
    'theatre-chat:' || sid::text,
    true
  );
end;
$$;
revoke all on function public.theatre_say(uuid, text, integer) from public, anon;
grant execute on function public.theatre_say(uuid, text, integer) to authenticated;

-- ── Who may listen and react (Realtime Authorization) ──────────────────────
--   theatre:<id>        presence (who's here) and reactions: in-screening people may send
--   theatre-chat:<id>   chat: in-screening people may only listen; theatre_say sends

create or replace function public.theatre_topic_id(topic text)
returns uuid
language sql
immutable
as $$
  select case when topic ~ '^theatre(-chat)?:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
              then substring(topic from '[0-9a-f-]{36}$')::uuid end
$$;

drop policy if exists "theatre: listen" on realtime.messages;
create policy "theatre: listen" on realtime.messages for select to authenticated
  using (public.theatre_can_enter(public.theatre_topic_id(realtime.topic()), (select auth.uid())));

drop policy if exists "theatre: react and be seen" on realtime.messages;
create policy "theatre: react and be seen" on realtime.messages for insert to authenticated
  with check (
    realtime.topic() like 'theatre:%'
    and realtime.messages.extension in ('broadcast', 'presence')
    and public.theatre_can_enter(public.theatre_topic_id(realtime.topic()), (select auth.uid()))
  );

-- ── The minute job ─────────────────────────────────────────────────────────

create or replace function public.theatre_tick()
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  s record;
  freed integer;
begin
  -- Bookings surge on an open screening: it becomes ticketed before it fills.
  update public.theatre_screenings
  set mode = 'fixed'
  where status = 'scheduled' and mode = 'open' and now() < starts_at and booked >= ceil(capacity * 0.8);

  -- 10 minutes in: seats of ticket holders who never came go to the waitlist.
  for s in select id from public.theatre_screenings
           where status = 'scheduled' and mode = 'fixed' and not released and now() >= starts_at + interval '10 minutes' loop
    update public.theatre_tickets set status = 'released'
    where screening_id = s.id and status = 'booked' and joined_at is null;
    get diagnostics freed = row_count;
    update public.theatre_screenings set booked = greatest(booked - freed, 0), released = true where id = s.id;
    perform public.theatre_promote(s.id);
  end loop;

  -- Chat closed: settle who came and who didn't, and forget the chat state.
  for s in select id from public.theatre_screenings
           where status = 'scheduled' and not settled and now() > ends_at + interval '30 minutes' loop
    update public.theatre_tickets set status = case when joined_at is not null then 'attended' else 'noshow' end
    where screening_id = s.id and status = 'booked';
    update public.theatre_tickets set status = case when joined_at is not null then 'attended' else status end
    where screening_id = s.id and status = 'released';
    update public.theatre_tickets set status = 'cancelled' where screening_id = s.id and status = 'waitlist';
    delete from public.theatre_chat_state where screening_id = s.id;
    update public.theatre_screenings set settled = true where id = s.id;
  end loop;

  delete from public.theatre_chat_rate where minute < now() - interval '1 hour';
end;
$$;
revoke all on function public.theatre_tick() from public, anon, authenticated;

do $$
begin
  if exists (select 1 from pg_extension where extname = 'pg_cron') then
    perform cron.unschedule(jobid) from cron.job where jobname = 'theatre-tick';
    perform cron.schedule('theatre-tick', '* * * * *', 'select public.theatre_tick()');
  else
    raise notice 'pg_cron is not enabled: enable it (Database > Extensions) and run this file again to start the Theatre minute job.';
  end if;
end;
$$;

-- ── Export includes your tickets ───────────────────────────────────────────

create or replace function public.export_my_theatre()
returns jsonb
language sql
stable
security definer
set search_path = ''
as $$
  select coalesce(jsonb_agg(jsonb_build_object('screening', s.name, 'starts_at', s.starts_at, 'status', t.status) order by s.starts_at desc), '[]'::jsonb)
  from public.theatre_tickets t join public.theatre_screenings s on s.id = t.screening_id
  where t.user_id = auth.uid();
$$;
revoke all on function public.export_my_theatre() from public, anon;
grant execute on function public.export_my_theatre() to authenticated;

-- ════════════════════════════════════════════════════════════════════════════
-- Phase 5: Theatre on Velmora Social. Booking and attending show in followers'
-- feeds (only while the person shares what they watch).
-- ════════════════════════════════════════════════════════════════════════════

alter table public.activity drop constraint if exists activity_kind_check;
alter table public.activity add constraint activity_kind_check
  check (kind in ('watching', 'finished', 'listed', 'rated', 'booked', 'attended'));

-- One feed line per person and title per day for a Theatre event.
create or replace function public.theatre_post(uid uuid, what text, title text)
returns void
language plpgsql
security definer
set search_path = ''
as $$
begin
  if not public.profile_sharing(uid) then return; end if;
  if exists (select 1 from public.activity a where a.user_id = uid and a.k = title and a.kind = what and a.created_at > now() - interval '1 day') then
    return;
  end if;
  insert into public.activity (user_id, kind, k) values (uid, what, title);
end;
$$;
revoke all on function public.theatre_post(uuid, text, text) from public, anon, authenticated;

-- Booking now also posts "booked".
create or replace function public.theatre_book(sid uuid)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  s public.theatre_screenings;
  current text;
  result text;
begin
  if me is null then raise exception 'sign_in' using errcode = 'P0001'; end if;
  if not public.profile_visible(me) then raise exception 'not_allowed' using errcode = 'P0001'; end if;
  select * into s from public.theatre_screenings where id = sid for update;
  if not found or s.status <> 'scheduled' then raise exception 'not_bookable' using errcode = 'P0001'; end if;
  if now() >= s.ends_at then raise exception 'over' using errcode = 'P0001'; end if;
  if s.opens_at is not null and now() < public.theatre_opens_for(s, me) then
    raise exception 'not_open_yet' using errcode = 'P0001';
  end if;

  select status into current from public.theatre_tickets where screening_id = sid and user_id = me;
  if current in ('booked', 'waitlist', 'attended') then return 'already'; end if;

  result := case when s.mode = 'fixed' and (s.booked >= s.capacity or s.released) then 'waitlist' else 'booked' end;
  insert into public.theatre_tickets (screening_id, user_id, status, is_max, created_at)
  values (sid, me, result, public.is_max(me), now())
  on conflict (screening_id, user_id) do update set status = excluded.status, is_max = excluded.is_max, created_at = now(), joined_at = null;

  if result = 'booked' then
    update public.theatre_screenings set booked = booked + 1 where id = sid;
    perform public.theatre_post(me, 'booked', s.k);
  else
    update public.theatre_screenings set waitlisted = waitlisted + 1 where id = sid;
  end if;
  return result;
end;
$$;
revoke all on function public.theatre_book(uuid) from public, anon;
grant execute on function public.theatre_book(uuid) to authenticated;

-- The minute job now also posts "attended" when a screening wraps up.
create or replace function public.theatre_tick()
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  s record;
  t record;
  freed integer;
begin
  update public.theatre_screenings
  set mode = 'fixed'
  where status = 'scheduled' and mode = 'open' and now() < starts_at and booked >= ceil(capacity * 0.8);

  for s in select id from public.theatre_screenings
           where status = 'scheduled' and mode = 'fixed' and not released and now() >= starts_at + interval '10 minutes' loop
    update public.theatre_tickets set status = 'released'
    where screening_id = s.id and status = 'booked' and joined_at is null;
    get diagnostics freed = row_count;
    update public.theatre_screenings set booked = greatest(booked - freed, 0), released = true where id = s.id;
    perform public.theatre_promote(s.id);
  end loop;

  for s in select id, k from public.theatre_screenings
           where status = 'scheduled' and not settled and now() > ends_at + interval '30 minutes' loop
    update public.theatre_tickets set status = case when joined_at is not null then 'attended' else 'noshow' end
    where screening_id = s.id and status = 'booked';
    update public.theatre_tickets set status = case when joined_at is not null then 'attended' else status end
    where screening_id = s.id and status = 'released';
    update public.theatre_tickets set status = 'cancelled' where screening_id = s.id and status = 'waitlist';
    for t in select user_id from public.theatre_tickets where screening_id = s.id and status = 'attended' loop
      perform public.theatre_post(t.user_id, 'attended', s.k);
    end loop;
    delete from public.theatre_chat_state where screening_id = s.id;
    update public.theatre_screenings set settled = true where id = s.id;
  end loop;

  delete from public.theatre_chat_rate where minute < now() - interval '1 hour';
end;
$$;
revoke all on function public.theatre_tick() from public, anon, authenticated;

-- ════════════════════════════════════════════════════════════════════════════
-- Phase 6: the admin site's hand on Theatre (docs/admin.md, phase 2).
-- Special airings, a pause switch, a banner, per-screening chat modes and
-- room mutes. Everything people can't change is written with the service key.
-- ════════════════════════════════════════════════════════════════════════════

-- Screenings the owner added by hand are "special": outside the daily lineup,
-- left alone by the algorithm. `note` is a short public message ("Delayed to
-- 9 pm"); `chat_mode` is normal, slow (10 s between messages) or closed.
alter table public.theatre_screenings add column if not exists special boolean not null default false;
alter table public.theatre_screenings add column if not exists created_by uuid references auth.users (id) on delete set null;
alter table public.theatre_screenings add column if not exists note text not null default '' check (char_length(note) <= 200);
alter table public.theatre_screenings add column if not exists chat_mode text not null default 'normal';
alter table public.theatre_screenings drop constraint if exists theatre_chat_mode_ok;
alter table public.theatre_screenings add constraint theatre_chat_mode_ok check (chat_mode in ('normal', 'slow', 'closed'));

-- One row of switches. Everyone may read it (the apps show the banner and the
-- pause notice); only the service key writes it.
create table if not exists public.theatre_settings (
  id boolean primary key default true check (id),
  paused boolean not null default false,
  banner text not null default '' check (char_length(banner) <= 200),
  -- Lobby reminder text; {title} becomes the film. Empty means the default.
  reminder text not null default '' check (char_length(reminder) <= 160),
  updated_at timestamptz not null default now()
);
insert into public.theatre_settings (id) values (true) on conflict do nothing;
alter table public.theatre_settings enable row level security;
drop policy if exists "theatre settings are public" on public.theatre_settings;
create policy "theatre settings are public" on public.theatre_settings for select to anon, authenticated using (true);

-- Muted people can watch but not chat; removed people are out of the room.
create table if not exists public.theatre_room_bans (
  screening_id uuid not null references public.theatre_screenings (id) on delete cascade,
  user_id uuid not null references public.profiles (id) on delete cascade,
  kind text not null check (kind in ('mute', 'remove')),
  created_at timestamptz not null default now(),
  primary key (screening_id, user_id)
);
alter table public.theatre_room_bans enable row level security;

-- Removed people may not enter (this also closes the Realtime topics).
create or replace function public.theatre_can_enter(sid uuid, uid uuid)
returns boolean
language sql
stable
security definer
set search_path = ''
as $$
  select exists (
    select 1 from public.theatre_screenings s
    where s.id = sid
      and public.theatre_window_open(s)
      and public.profile_visible(uid)
      and not exists (select 1 from public.theatre_room_bans b where b.screening_id = s.id and b.user_id = uid and b.kind = 'remove')
      and (s.mode = 'open'
           or exists (select 1 from public.theatre_tickets t
                      where t.screening_id = s.id and t.user_id = uid and t.status in ('booked', 'attended')))
  )
$$;

-- Chat now honours the room's mode and mutes.
create or replace function public.theatre_say(sid uuid, body text, movie_ms integer default null)
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  p public.profiles;
  msg text := btrim(regexp_replace(coalesce(body, ''), '\s+', ' ', 'g'));
  room text;
  busy integer;
  gap interval;
  st public.theatre_chat_state;
  h text;
  w text;
begin
  if me is null then raise exception 'sign_in' using errcode = 'P0001'; end if;
  if not public.theatre_can_enter(sid, me) then raise exception 'not_in_screening' using errcode = 'P0001'; end if;
  select chat_mode into room from public.theatre_screenings where id = sid;
  if room = 'closed' then raise exception 'chat_closed' using errcode = 'P0001'; end if;
  if exists (select 1 from public.theatre_room_bans b where b.screening_id = sid and b.user_id = me and b.kind = 'mute') then
    raise exception 'chat_muted' using errcode = 'P0001';
  end if;
  if char_length(msg) not between 1 and 200 then raise exception 'chat_length' using errcode = 'P0001'; end if;
  if msg ~* '(https?://|www\.|[a-z0-9-]+\.(com|net|org|io|gg|xyz|ru|me|tv|co|ly|link)\b)' then
    raise exception 'chat_link' using errcode = 'P0001';
  end if;
  for w in select b.word from public.blocked_words b loop
    if msg ~* ('(^|[^a-z0-9])' || regexp_replace(w, '([.*+?^${}()|\[\]\\])', '\\\1', 'g') || '($|[^a-z0-9])') then
      raise exception 'chat_blocked' using errcode = 'P0001';
    end if;
  end loop;

  select coalesce(sum(r.n), 0) into busy from public.theatre_chat_rate r
  where r.screening_id = sid and r.minute > now() - interval '1 minute';
  gap := case when busy > 300 then interval '10 seconds' when busy > 120 then interval '5 seconds'
              when busy > 40 then interval '2.5 seconds' else interval '1 second' end;
  if room = 'slow' and gap < interval '10 seconds' then gap := interval '10 seconds'; end if;
  h := md5(lower(msg));
  select * into st from public.theatre_chat_state where screening_id = sid and user_id = me;
  if found then
    if st.last_at > now() - gap then raise exception 'chat_slow' using errcode = 'P0001', detail = ceil(extract(epoch from (st.last_at + gap - now())))::text; end if;
    if st.last_hash = h and st.last_at > now() - interval '60 seconds' then raise exception 'chat_repeat' using errcode = 'P0001'; end if;
  end if;
  insert into public.theatre_chat_state (screening_id, user_id, last_at, last_hash) values (sid, me, now(), h)
    on conflict (screening_id, user_id) do update set last_at = now(), last_hash = h;
  insert into public.theatre_chat_rate (screening_id, minute, n) values (sid, date_trunc('minute', now()), 1)
    on conflict (screening_id, minute) do update set n = public.theatre_chat_rate.n + 1;

  select * into p from public.profiles where id = me;
  perform realtime.send(
    jsonb_build_object(
      'id', gen_random_uuid(),
      'uid', p.id, 'username', p.username, 'display_name', p.display_name, 'avatar', p.avatar, 'verified', p.verified,
      'body', msg,
      'at', movie_ms,
      'ts', now()
    ),
    'chat',
    'theatre-chat:' || sid::text,
    true
  );
end;
$$;
revoke all on function public.theatre_say(uuid, text, integer) from public, anon;
grant execute on function public.theatre_say(uuid, text, integer) to authenticated;

-- Booking stops while Theatre is paused (people already in a show stay in).
create or replace function public.theatre_book(sid uuid)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  s public.theatre_screenings;
  current text;
  result text;
begin
  if me is null then raise exception 'sign_in' using errcode = 'P0001'; end if;
  if exists (select 1 from public.theatre_settings t where t.paused) then raise exception 'paused' using errcode = 'P0001'; end if;
  if not public.profile_visible(me) then raise exception 'not_allowed' using errcode = 'P0001'; end if;
  select * into s from public.theatre_screenings where id = sid for update;
  if not found or s.status <> 'scheduled' then raise exception 'not_bookable' using errcode = 'P0001'; end if;
  if now() >= s.ends_at then raise exception 'over' using errcode = 'P0001'; end if;
  if s.opens_at is not null and now() < public.theatre_opens_for(s, me) then
    raise exception 'not_open_yet' using errcode = 'P0001';
  end if;

  select status into current from public.theatre_tickets where screening_id = sid and user_id = me;
  if current in ('booked', 'waitlist', 'attended') then return 'already'; end if;

  result := case when s.mode = 'fixed' and (s.booked >= s.capacity or s.released) then 'waitlist' else 'booked' end;
  insert into public.theatre_tickets (screening_id, user_id, status, is_max, created_at)
  values (sid, me, result, public.is_max(me), now())
  on conflict (screening_id, user_id) do update set status = excluded.status, is_max = excluded.is_max, created_at = now(), joined_at = null;

  if result = 'booked' then
    update public.theatre_screenings set booked = booked + 1 where id = sid;
    perform public.theatre_post(me, 'booked', s.k);
  else
    update public.theatre_screenings set waitlisted = waitlisted + 1 where id = sid;
  end if;
  return result;
end;
$$;
revoke all on function public.theatre_book(uuid) from public, anon;
grant execute on function public.theatre_book(uuid) to authenticated;

-- ── Admin functions (service key only) ─────────────────────────────────────

-- Cancel a screening. Tickets stay as they are, so people who held one can be
-- told it was cancelled; a cancelled screening is never settled, so nobody
-- gets a no-show for it.
create or replace function public.admin_theatre_cancel(sid uuid, why text default '')
returns void
language plpgsql
security definer
set search_path = ''
as $$
begin
  update public.theatre_screenings
  set status = 'cancelled', note = left(coalesce(nullif(btrim(why), ''), note), 200)
  where id = sid and status in ('draft', 'scheduled');
  if not found then raise exception 'not_cancellable' using errcode = 'P0001'; end if;
end;
$$;

-- Give someone a seat whether or not it is full.
create or replace function public.admin_theatre_grant(sid uuid, uid uuid)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  s public.theatre_screenings;
  cur text;
begin
  select * into s from public.theatre_screenings where id = sid for update;
  if not found or s.status <> 'scheduled' then raise exception 'not_bookable' using errcode = 'P0001'; end if;
  if now() >= s.ends_at then raise exception 'over' using errcode = 'P0001'; end if;
  select status into cur from public.theatre_tickets where screening_id = sid and user_id = uid;
  if cur in ('booked', 'attended') then return 'already'; end if;
  insert into public.theatre_tickets (screening_id, user_id, status, is_max, created_at)
  values (sid, uid, 'booked', public.is_max(uid), now())
  on conflict (screening_id, user_id) do update set status = 'booked', is_max = excluded.is_max, created_at = now();
  update public.theatre_screenings
  set booked = booked + 1, waitlisted = greatest(waitlisted - (case when cur = 'waitlist' then 1 else 0 end), 0)
  where id = sid;
  return 'booked';
end;
$$;

-- Take a seat or a waitlist place away; a freed seat goes to the waitlist.
create or replace function public.admin_theatre_revoke(sid uuid, uid uuid)
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  cur text;
begin
  perform 1 from public.theatre_screenings where id = sid for update;
  select status into cur from public.theatre_tickets where screening_id = sid and user_id = uid;
  if cur not in ('booked', 'waitlist') then return; end if;
  update public.theatre_tickets set status = 'cancelled' where screening_id = sid and user_id = uid;
  if cur = 'booked' then
    update public.theatre_screenings set booked = greatest(booked - 1, 0) where id = sid;
    perform public.theatre_promote(sid);
  else
    update public.theatre_screenings set waitlisted = greatest(waitlisted - 1, 0) where id = sid;
  end if;
end;
$$;

-- Set the room's chat mode and tell everyone in it right away.
create or replace function public.admin_theatre_chat(sid uuid, mode text)
returns void
language plpgsql
security definer
set search_path = ''
as $$
begin
  if mode not in ('normal', 'slow', 'closed') then raise exception 'bad_mode' using errcode = 'P0001'; end if;
  update public.theatre_screenings set chat_mode = mode where id = sid;
  if not found then raise exception 'not_found' using errcode = 'P0001'; end if;
  perform realtime.send(jsonb_build_object('mode', mode), 'mode', 'theatre-chat:' || sid::text, true);
end;
$$;

-- Mute, remove or restore one person in one room.
create or replace function public.admin_theatre_room(sid uuid, uid uuid, what text)
returns void
language plpgsql
security definer
set search_path = ''
as $$
begin
  if what = 'clear' then
    delete from public.theatre_room_bans where screening_id = sid and user_id = uid;
  elsif what in ('mute', 'remove') then
    insert into public.theatre_room_bans (screening_id, user_id, kind) values (sid, uid, what)
    on conflict (screening_id, user_id) do update set kind = excluded.kind, created_at = now();
  else
    raise exception 'bad_kind' using errcode = 'P0001';
  end if;
end;
$$;

revoke all on function public.admin_theatre_cancel(uuid, text) from public, anon, authenticated;
revoke all on function public.admin_theatre_grant(uuid, uuid) from public, anon, authenticated;
revoke all on function public.admin_theatre_revoke(uuid, uuid) from public, anon, authenticated;
revoke all on function public.admin_theatre_chat(uuid, text) from public, anon, authenticated;
revoke all on function public.admin_theatre_room(uuid, uuid, text) from public, anon, authenticated;
grant execute on function public.admin_theatre_cancel(uuid, text) to service_role;
grant execute on function public.admin_theatre_grant(uuid, uuid) to service_role;
grant execute on function public.admin_theatre_revoke(uuid, uuid) to service_role;
grant execute on function public.admin_theatre_chat(uuid, text) to service_role;
grant execute on function public.admin_theatre_room(uuid, uuid, text) to service_role;
