-- Velmora social groundwork: animal avatars, bios and links, follows, profile
-- visits, reports, and the admin-only tables.
--
-- Run in Supabase's SQL Editor AFTER schema.sql and sync.sql. Safe to run again.

-- ── Profiles: new fields ───────────────────────────────────────────────────

alter table public.profiles add column if not exists bio text not null default '';
alter table public.profiles add column if not exists links jsonb not null default '[]'::jsonb;
-- Set only by the admin site (server code with the service key), never by the person.
alter table public.profiles add column if not exists verified boolean not null default false;
alter table public.profiles add column if not exists hidden boolean not null default false;
alter table public.profiles add column if not exists banned boolean not null default false;
-- Watching activity on the social site: shared unless switched off (phase 3).
alter table public.profiles add column if not exists share_activity boolean not null default true;
-- Kept up to date by triggers on follows, so profile pages don't count rows.
alter table public.profiles add column if not exists followers_count integer not null default 0;
alter table public.profiles add column if not exists following_count integer not null default 0;

-- ── Avatars: the nine animals replace the fox presets ──────────────────────

alter table public.profiles drop constraint if exists profiles_avatar_ok;

update public.profiles
set avatar = (array['panda','owl','rabbit','chameleon','octopus','dragon','tiger','wolf','cat'])[
  ((substring(avatar from 8)::int - 1) % 9) + 1]
where avatar ~ '^preset:[0-9]{1,2}$';
update public.profiles set avatar = 'animal:' || avatar where avatar in ('panda','owl','rabbit','chameleon','octopus','dragon','tiger','wolf','cat');

alter table public.profiles alter column avatar set default 'animal:panda';
alter table public.profiles add constraint profiles_avatar_ok check (
  char_length(avatar) <= 500
  and (
    avatar ~ '^animal:(panda|owl|rabbit|chameleon|octopus|dragon|tiger|wolf|cat)$'
    or avatar ~ '^https://(lh3\.googleusercontent\.com|cdn\.discordapp\.com)/[^[:space:]]+$'
  )
);

-- ── Bio and links ──────────────────────────────────────────────────────────

-- Links are [{ "kind": "instagram", "url": "https://instagram.com/..." }], at
-- most six, each pointing at the site its kind names ("website" may be any https page).
create or replace function public.valid_links(links jsonb)
returns boolean
language plpgsql
immutable
set search_path = ''
as $$
declare
  item jsonb;
  kind text;
  url text;
  hosts constant jsonb := '{
    "instagram": "instagram\\.com",
    "x": "(x|twitter)\\.com",
    "youtube": "(youtube\\.com|youtu\\.be)",
    "tiktok": "tiktok\\.com",
    "twitch": "twitch\\.tv",
    "github": "github\\.com",
    "discord": "(discord\\.gg|discord\\.com)",
    "letterboxd": "letterboxd\\.com",
    "anilist": "anilist\\.co",
    "myanimelist": "myanimelist\\.net",
    "website": "[a-z0-9.-]+\\.[a-z]{2,}"
  }';
begin
  if jsonb_typeof(links) <> 'array' or jsonb_array_length(links) > 6 then return false; end if;
  for item in select value from jsonb_array_elements(links) loop
    if jsonb_typeof(item) <> 'object' then return false; end if;
    kind := item ->> 'kind';
    url := item ->> 'url';
    if kind is null or url is null or not (hosts ? kind) or char_length(url) > 200 then return false; end if;
    if url !~* ('^https://([a-z0-9-]+\.)*' || (hosts ->> kind) || '(/[^[:space:]<>"]*)?$') then return false; end if;
  end loop;
  return true;
end;
$$;

alter table public.profiles drop constraint if exists profiles_bio_len;
alter table public.profiles add constraint profiles_bio_len check (char_length(bio) <= 160);
alter table public.profiles drop constraint if exists profiles_links_ok;
alter table public.profiles add constraint profiles_links_ok check (public.valid_links(links));

-- ── The guard, now also protecting what only the admin site may set ────────

create or replace function public.profiles_guard()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
declare
  staff boolean := coalesce(auth.jwt() ->> 'role', '') = 'service_role';
begin
  if tg_op = 'INSERT' then
    if exists (select 1 from public.reserved_usernames r where r.name = lower(new.username)) then
      raise exception 'username_reserved' using errcode = 'P0001';
    end if;
    new.username_changed_at := null;
    new.created_at := now();
    if not staff then
      new.verified := false;
      new.hidden := false;
      new.banned := false;
    end if;
    new.followers_count := 0;
    new.following_count := 0;
  else
    new.id := old.id;
    new.created_at := old.created_at;
    if not staff then
      new.verified := old.verified;
      new.hidden := old.hidden;
      new.banned := old.banned;
      -- Counts move only through follows (the trigger there runs as staff-like definer).
      if current_setting('velmora.counting', true) is distinct from 'on' then
        new.followers_count := old.followers_count;
        new.following_count := old.following_count;
      end if;
    end if;
    if lower(new.username) <> lower(old.username) then
      if exists (select 1 from public.reserved_usernames r where r.name = lower(new.username)) then
        raise exception 'username_reserved' using errcode = 'P0001';
      end if;
      if not staff and old.username_changed_at is not null and old.username_changed_at > now() - interval '30 days' then
        raise exception 'username_cooldown' using errcode = 'P0001';
      end if;
      new.username_changed_at := now();
    else
      new.username_changed_at := old.username_changed_at;
    end if;
  end if;
  new.updated_at := now();
  return new;
end;
$$;

-- Profiles are public on the social site, except hidden and banned ones (their
-- owner still sees their own). Email is never in this table.
drop policy if exists "profiles are visible to signed-in users" on public.profiles;
drop policy if exists "public profiles" on public.profiles;
create policy "public profiles" on public.profiles for select to anon, authenticated
  using ((not hidden and not banned) or id = (select auth.uid()));

-- ── Follows ────────────────────────────────────────────────────────────────

create table if not exists public.follows (
  follower uuid not null references public.profiles (id) on delete cascade,
  followee uuid not null references public.profiles (id) on delete cascade,
  created_at timestamptz not null default now(),
  primary key (follower, followee),
  constraint follows_not_self check (follower <> followee)
);
create index if not exists follows_followee on public.follows (followee, created_at desc);
create index if not exists follows_follower on public.follows (follower, created_at desc);

alter table public.follows enable row level security;
drop policy if exists "follows are public" on public.follows;
create policy "follows are public" on public.follows for select to anon, authenticated using (true);
drop policy if exists "follow as yourself" on public.follows;
create policy "follow as yourself" on public.follows for insert to authenticated with check ((select auth.uid()) = follower);
drop policy if exists "unfollow as yourself" on public.follows;
create policy "unfollow as yourself" on public.follows for delete to authenticated using ((select auth.uid()) = follower);

create or replace function public.follows_count()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  perform set_config('velmora.counting', 'on', true);
  if tg_op = 'INSERT' then
    update public.profiles set followers_count = followers_count + 1 where id = new.followee;
    update public.profiles set following_count = following_count + 1 where id = new.follower;
  else
    update public.profiles set followers_count = greatest(followers_count - 1, 0) where id = old.followee;
    update public.profiles set following_count = greatest(following_count - 1, 0) where id = old.follower;
  end if;
  perform set_config('velmora.counting', 'off', true);
  return null;
end;
$$;

drop trigger if exists follows_count on public.follows;
create trigger follows_count after insert or delete on public.follows
  for each row execute function public.follows_count();

-- ── Profile visits: the count is public, who visited is the owner's alone ──

create table if not exists public.profile_visits_daily (
  profile_id uuid not null references public.profiles (id) on delete cascade,
  day date not null,
  visits integer not null default 0,
  primary key (profile_id, day)
);
alter table public.profile_visits_daily enable row level security;
drop policy if exists "visit counts are public" on public.profile_visits_daily;
create policy "visit counts are public" on public.profile_visits_daily for select to anon, authenticated using (true);

create table if not exists public.profile_visitors (
  profile_id uuid not null references public.profiles (id) on delete cascade,
  visitor_id uuid not null references public.profiles (id) on delete cascade,
  last_at timestamptz not null default now(),
  primary key (profile_id, visitor_id)
);
create index if not exists profile_visitors_recent on public.profile_visitors (profile_id, last_at desc);
alter table public.profile_visitors enable row level security;
drop policy if exists "owners see their visitors" on public.profile_visitors;
create policy "owners see their visitors" on public.profile_visitors for select to authenticated
  using ((select auth.uid()) = profile_id);

-- Record a visit. Your own visits don't count; a signed-in visitor counts once a day.
create or replace function public.record_visit(target uuid)
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  seen timestamptz;
begin
  if target is null or target = me then return; end if;
  if not exists (select 1 from public.profiles p where p.id = target and not p.hidden and not p.banned) then return; end if;
  if me is not null then
    select last_at into seen from public.profile_visitors where profile_id = target and visitor_id = me;
    insert into public.profile_visitors (profile_id, visitor_id, last_at) values (target, me, now())
      on conflict (profile_id, visitor_id) do update set last_at = now();
    if seen is not null and seen::date = now()::date then return; end if;
  end if;
  insert into public.profile_visits_daily (profile_id, day, visits) values (target, now()::date, 1)
    on conflict (profile_id, day) do update set visits = public.profile_visits_daily.visits + 1;
end;
$$;
revoke all on function public.record_visit(uuid) from public;
grant execute on function public.record_visit(uuid) to anon, authenticated;

-- Visits in the last 30 days, for the public count.
create or replace function public.visits_this_month(target uuid)
returns integer
language sql
stable
security definer
set search_path = ''
as $$
  select coalesce(sum(visits), 0)::int from public.profile_visits_daily
  where profile_id = target and day > (now()::date - 30);
$$;
revoke all on function public.visits_this_month(uuid) from public;
grant execute on function public.visits_this_month(uuid) to anon, authenticated;

-- ── Reports ────────────────────────────────────────────────────────────────

create table if not exists public.reports (
  id bigint generated always as identity primary key,
  reporter uuid not null references public.profiles (id) on delete cascade,
  target uuid not null references public.profiles (id) on delete cascade,
  reason text not null check (reason in ('username', 'display_name', 'bio', 'links', 'avatar', 'review', 'spam', 'other')),
  note text not null default '' check (char_length(note) <= 500),
  status text not null default 'open' check (status in ('open', 'dismissed', 'actioned')),
  created_at timestamptz not null default now()
);
create index if not exists reports_open on public.reports (status, created_at);
alter table public.reports enable row level security;
-- People file reports; only the admin site reads them.
drop policy if exists "report as yourself" on public.reports;
create policy "report as yourself" on public.reports for insert to authenticated
  with check ((select auth.uid()) = reporter and reporter <> target and status = 'open');

-- ── Admin-only tables (no policies: only server code with the service key) ─

create table if not exists public.admins (
  user_id uuid primary key references auth.users (id) on delete cascade,
  added_at timestamptz not null default now()
);
alter table public.admins enable row level security;

create table if not exists public.audit_log (
  id bigint generated always as identity primary key,
  admin_id uuid references auth.users (id) on delete set null,
  action text not null,
  target text,
  details jsonb not null default '{}'::jsonb,
  at timestamptz not null default now()
);
alter table public.audit_log enable row level security;

create table if not exists public.blocked_words (
  word text primary key check (word = lower(word) and char_length(word) between 2 and 40)
);
alter table public.blocked_words enable row level security;

-- ── Export now includes the social side ────────────────────────────────────

create or replace function public.export_my_data()
returns jsonb
language sql
stable
security definer
set search_path = ''
as $$
  select jsonb_build_object(
    'exported_at', now(),
    'account', jsonb_build_object(
      'id', u.id,
      'email', u.email,
      'created_at', u.created_at,
      'providers', u.raw_app_meta_data -> 'providers'
    ),
    'profile', to_jsonb(p) - 'username_key',
    'following', coalesce((select jsonb_agg(f.username) from public.follows x join public.profiles f on f.id = x.followee where x.follower = u.id), '[]'::jsonb),
    'my_list', coalesce((select jsonb_agg(l.title order by l.updated_at desc)
                         from public.list_items l where l.user_id = u.id and not l.deleted), '[]'::jsonb),
    'progress', coalesce((select jsonb_agg(g.entry order by g.updated_at desc)
                          from public.progress g where g.user_id = u.id and not g.deleted), '[]'::jsonb),
    'settings', (select s.data from public.user_settings s where s.user_id = u.id)
  )
  from auth.users u
  left join public.profiles p on p.id = u.id
  where u.id = auth.uid();
$$;
