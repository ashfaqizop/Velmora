-- Velmora admin site (admin.getvelmora.dpdns.org), phase 1.
-- Run after social.sql and activity.sql. Safe to run again.
--
-- Who is an admin: the `admins` table from social.sql. Add yourself once, in the
-- SQL editor, after you have signed up (use your own sign-in email):
--
--   insert into public.admins (user_id)
--   select id from auth.users where email = 'you@example.com'
--   on conflict do nothing;
--
-- The admin site checks this table AND that the sign-in passed TOTP (aal2).
-- Everything below is callable only by the service key, which lives in Vercel.

-- ── Reports remember who handled them and when ─────────────────────────────

alter table public.reports add column if not exists handled_at timestamptz;
alter table public.reports add column if not exists handled_by uuid references auth.users (id) on delete set null;

-- ── Blocked words now also guard what people write on profiles ─────────────
-- Theatre chat already used the list. `strict` matches inside words too (for
-- usernames, where there are no spaces to split on).

create or replace function public.has_blocked_word(txt text, strict boolean default false)
returns boolean
language plpgsql
stable
security definer
set search_path = ''
as $$
declare
  w text;
  esc text;
begin
  if txt is null or txt = '' then return false; end if;
  for w in select b.word from public.blocked_words b loop
    esc := regexp_replace(w, '([.*+?^${}()|\[\]\\])', '\\\1', 'g');
    if strict then
      if txt ~* esc then return true; end if;
    elsif txt ~* ('(^|[^a-z0-9])' || esc || '($|[^a-z0-9])') then
      return true;
    end if;
  end loop;
  return false;
end;
$$;
revoke all on function public.has_blocked_word(text, boolean) from public, anon, authenticated;

create or replace function public.profiles_words()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if coalesce(auth.jwt() ->> 'role', '') = 'service_role' then return new; end if;
  if (tg_op = 'INSERT' or new.username is distinct from old.username) and public.has_blocked_word(new.username, true) then
    raise exception 'blocked_word' using errcode = 'P0001';
  end if;
  if (tg_op = 'INSERT' or new.display_name is distinct from old.display_name) and public.has_blocked_word(new.display_name) then
    raise exception 'blocked_word' using errcode = 'P0001';
  end if;
  if (tg_op = 'INSERT' or new.bio is distinct from old.bio) and public.has_blocked_word(new.bio) then
    raise exception 'blocked_word' using errcode = 'P0001';
  end if;
  return new;
end;
$$;
drop trigger if exists profiles_words on public.profiles;
create trigger profiles_words before insert or update on public.profiles
  for each row execute function public.profiles_words();

create or replace function public.ratings_words()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if coalesce(auth.jwt() ->> 'role', '') = 'service_role' then return new; end if;
  if (tg_op = 'INSERT' or new.review is distinct from old.review) and public.has_blocked_word(new.review) then
    raise exception 'blocked_word' using errcode = 'P0001';
  end if;
  return new;
end;
$$;
drop trigger if exists ratings_words on public.ratings;
create trigger ratings_words before insert or update on public.ratings
  for each row execute function public.ratings_words();

create or replace function public.lists_words()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if coalesce(auth.jwt() ->> 'role', '') = 'service_role' then return new; end if;
  if (tg_op = 'INSERT' or new.name is distinct from old.name) and public.has_blocked_word(new.name) then
    raise exception 'blocked_word' using errcode = 'P0001';
  end if;
  if (tg_op = 'INSERT' or new.description is distinct from old.description) and public.has_blocked_word(new.description) then
    raise exception 'blocked_word' using errcode = 'P0001';
  end if;
  return new;
end;
$$;
drop trigger if exists lists_words on public.lists;
create trigger lists_words before insert or update on public.lists
  for each row execute function public.lists_words();

-- ── Numbers for the overview ───────────────────────────────────────────────

create or replace function public.admin_overview()
returns jsonb
language sql
stable
security definer
set search_path = ''
as $$
  select jsonb_build_object(
    'db_bytes', pg_database_size(current_database()),
    'users', (select count(*) from auth.users),
    'signups_today', (select count(*) from auth.users where created_at >= date_trunc('day', now())),
    'signups_7d', (select count(*) from auth.users where created_at > now() - interval '7 days'),
    'active_30d', (select count(*) from auth.users where last_sign_in_at > now() - interval '30 days'),
    'profiles', (select count(*) from public.profiles),
    'hidden', (select count(*) from public.profiles where hidden),
    'banned', (select count(*) from public.profiles where banned),
    'verified', (select count(*) from public.profiles where verified),
    'reports_open', (select count(*) from public.reports where status = 'open'),
    'ratings', (select count(*) from public.ratings),
    'reviews', (select count(*) from public.ratings where review <> ''),
    'lists', (select count(*) from public.lists),
    'follows', (select count(*) from public.follows),
    'activity', (select count(*) from public.activity)
  );
$$;

-- ── Finding people (email lives in auth.users, so it takes a function) ─────

create or replace function public.admin_users(q text default '', lim integer default 30, off integer default 0)
returns table (
  id uuid, email text, username text, display_name text, avatar text,
  verified boolean, hidden boolean, banned boolean, followers_count integer,
  created_at timestamptz, last_sign_in_at timestamptz, reports_open bigint, total bigint
)
language plpgsql
stable
security definer
set search_path = ''
as $$
declare
  needle text := btrim(coalesce(q, ''));
  pat text;
begin
  pat := '%' || replace(replace(replace(needle, '\', '\\'), '%', '\%'), '_', '\_') || '%';
  return query
  select u.id, u.email::text, p.username, p.display_name, p.avatar,
         coalesce(p.verified, false), coalesce(p.hidden, false), coalesce(p.banned, false),
         coalesce(p.followers_count, 0), u.created_at, u.last_sign_in_at,
         (select count(*) from public.reports r where r.target = u.id and r.status = 'open'),
         count(*) over ()
  from auth.users u
  left join public.profiles p on p.id = u.id
  where needle = ''
     or u.id::text = lower(needle)
     or u.email ilike pat
     or p.username ilike pat
     or p.display_name ilike pat
  order by u.created_at desc
  limit least(greatest(lim, 1), 100) offset greatest(off, 0);
end;
$$;

-- One person, for the detail panel. Counts only: no watch history, no library,
-- nothing that was synced, because the admin never needs to see that.
create or replace function public.admin_user(uid uuid)
returns jsonb
language sql
stable
security definer
set search_path = ''
as $$
  select jsonb_build_object(
    'id', u.id,
    'email', u.email,
    'created_at', u.created_at,
    'last_sign_in_at', u.last_sign_in_at,
    'confirmed', u.email_confirmed_at is not null,
    'providers', coalesce((select array_agg(i.provider order by i.provider) from auth.identities i where i.user_id = u.id), '{}'),
    'profile', (select to_jsonb(p) - 'favorites' from public.profiles p where p.id = u.id),
    'ratings', (select count(*) from public.ratings r where r.user_id = u.id),
    'reviews', (select count(*) from public.ratings r where r.user_id = u.id and r.review <> ''),
    'lists', (select count(*) from public.lists l where l.user_id = u.id),
    'following', (select count(*) from public.follows f where f.follower = u.id),
    'followers', (select count(*) from public.follows f where f.followee = u.id),
    'reports_against', (select count(*) from public.reports r where r.target = u.id),
    'reports_open', (select count(*) from public.reports r where r.target = u.id and r.status = 'open'),
    'reports_filed', (select count(*) from public.reports r where r.reporter = u.id),
    'admin', exists (select 1 from public.admins a where a.user_id = u.id)
  )
  from auth.users u
  where u.id = uid;
$$;

revoke all on function public.admin_overview() from public, anon, authenticated;
revoke all on function public.admin_users(text, integer, integer) from public, anon, authenticated;
revoke all on function public.admin_user(uuid) from public, anon, authenticated;
grant execute on function public.admin_overview() to service_role;
grant execute on function public.admin_users(text, integer, integer) to service_role;
grant execute on function public.admin_user(uuid) to service_role;

-- ═══ Phase 3a: announcements and the maintenance message ══════════════════
-- Written only by the admin site (service key). Anyone, signed in or not, can
-- read the ones that are switched on and inside their time window: the app,
-- the website and Social show them.
--
--   kind      info (blue), warning (amber), maintenance (red, can't be dismissed)
--   audience  everyone, public (no Max), max
--   place     app, web, both

create table if not exists public.announcements (
  id          bigint generated always as identity primary key,
  kind        text not null default 'info' check (kind in ('info', 'warning', 'maintenance')),
  title       text not null check (char_length(title) between 1 and 80),
  body        text not null default '' check (char_length(body) <= 400),
  link_url    text not null default '' check (link_url = '' or link_url ~ '^https://[^\s]{3,300}$'),
  link_label  text not null default '' check (char_length(link_label) <= 30),
  audience    text not null default 'everyone' check (audience in ('everyone', 'public', 'max')),
  place       text not null default 'both' check (place in ('app', 'web', 'both')),
  active      boolean not null default true,
  starts_at   timestamptz not null default now(),
  ends_at     timestamptz,
  created_by  uuid references auth.users (id) on delete set null,
  created_at  timestamptz not null default now(),
  updated_at  timestamptz not null default now(),
  check (ends_at is null or ends_at > starts_at)
);

create index if not exists announcements_live on public.announcements (active, starts_at);

alter table public.announcements enable row level security;

drop policy if exists "live announcements are public" on public.announcements;
create policy "live announcements are public" on public.announcements
  for select to anon, authenticated
  using (active and starts_at <= now() and (ends_at is null or ends_at > now()));

revoke insert, update, delete on public.announcements from anon, authenticated;
-- Column grant: who wrote it stays private.
revoke select on public.announcements from anon, authenticated;
grant select (id, kind, title, body, link_url, link_label, audience, place, starts_at, ends_at, updated_at)
  on public.announcements to anon, authenticated;

-- ═══ Phase 3b: curation and growth numbers ════════════════════════════════
-- Social shows featured people and staff picks; the admin site picks them.
-- (Leaderboard hiding lives in activity.sql, next to the leaderboard.)

create table if not exists public.featured (
  id         bigint generated always as identity primary key,
  kind       text not null check (kind in ('person', 'title')),
  user_id    uuid references public.profiles (id) on delete cascade,
  k          text check (k is null or public.valid_title_key(k)),
  name       text not null default '' check (char_length(name) <= 200),
  poster     text not null default '' check (poster = '' or poster ~ '^https://image\.tmdb\.org/'),
  note       text not null default '' check (char_length(note) <= 140),
  position   integer not null default 0,
  created_at timestamptz not null default now(),
  check ((kind = 'person' and user_id is not null and k is null) or (kind = 'title' and k is not null and user_id is null))
);
create unique index if not exists featured_person on public.featured (user_id) where kind = 'person';
create unique index if not exists featured_title on public.featured (k) where kind = 'title';

alter table public.featured enable row level security;
drop policy if exists "staff picks are public" on public.featured;
create policy "staff picks are public" on public.featured for select to anon, authenticated using (kind = 'title');
revoke insert, update, delete on public.featured from anon, authenticated;

-- Featured people, with what a card needs; hidden or banned accounts drop out.
create or replace function public.featured_people()
returns table (id uuid, username text, display_name text, avatar text, verified boolean, followers_count integer, note text)
language sql
stable
security definer
set search_path = ''
as $$
  select p.id, p.username, p.display_name, p.avatar, p.verified, p.followers_count, f.note
  from public.featured f
  join public.profiles p on p.id = f.user_id
  where f.kind = 'person' and not p.hidden and not p.banned
  order by f.position, f.created_at
  limit 24;
$$;
grant execute on function public.featured_people() to anon, authenticated;

-- Day by day: sign-ups, follows, ratings, reviews and activity rows.
create or replace function public.admin_growth(days integer default 30)
returns table (day date, signups integer, follows integer, ratings integer, reviews integer, activity integer)
language sql
stable
security definer
set search_path = ''
as $$
  with d as (
    select generate_series((now() at time zone 'utc')::date - (least(greatest(days, 1), 120) - 1), (now() at time zone 'utc')::date, interval '1 day')::date as day
  )
  select d.day,
    (select count(*)::int from auth.users u where (u.created_at at time zone 'utc')::date = d.day),
    (select count(*)::int from public.follows f where (f.created_at at time zone 'utc')::date = d.day),
    (select count(*)::int from public.ratings r where (r.created_at at time zone 'utc')::date = d.day),
    (select count(*)::int from public.ratings r where r.review <> '' and (r.created_at at time zone 'utc')::date = d.day),
    (select count(*)::int from public.activity a where (a.created_at at time zone 'utc')::date = d.day)
  from d
  order by d.day;
$$;
revoke all on function public.admin_growth(integer) from public, anon, authenticated;
grant execute on function public.admin_growth(integer) to service_role;
