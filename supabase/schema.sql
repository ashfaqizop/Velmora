-- Velmora accounts: the profiles table, its rules, and the helper functions.
--
-- Run this once in Supabase: SQL Editor > New query > paste > Run.
-- It is safe to run again; everything is "create or replace" or "if not exists".
--
-- What lives where:
--   auth.users          Supabase's own table: email, provider, password-free logins.
--                       Never exposed to other people.
--   public.profiles     What other people can see: username, display name, avatar.
--
-- Nothing here stores a secret. The app and the website only ever hold the
-- public (publishable) key; every table is protected by row level security.

-- ── Reserved names ─────────────────────────────────────────────────────────

create table if not exists public.reserved_usernames (
  name text primary key check (name = lower(name))
);

-- No policies on purpose: only the functions below read it.
alter table public.reserved_usernames enable row level security;

insert into public.reserved_usernames (name) values
  ('admin'), ('administrator'), ('root'), ('system'), ('support'), ('help'), ('staff'),
  ('mod'), ('moderator'), ('velmora'), ('velmoraapp'), ('official'), ('team'), ('owner'),
  ('ani'), ('null'), ('undefined'), ('anonymous'), ('guest'), ('everyone'), ('here'),
  ('me'), ('you'), ('api'), ('www'), ('mail'), ('security'), ('billing'), ('contact'),
  ('info'), ('dev'), ('developer'), ('bot'), ('universeapi'), ('max'), ('public'),
  ('settings'), ('login'), ('logout'), ('signin'), ('signup'), ('auth'), ('account'),
  ('profile'), ('user'), ('users'), ('theatre'), ('theater'), ('chat'), ('siliconcode')
on conflict do nothing;

-- ── Profiles ───────────────────────────────────────────────────────────────

create table if not exists public.profiles (
  id uuid primary key references auth.users (id) on delete cascade,
  -- As typed, so "Azam" stays "Azam". Uniqueness ignores case (username_key).
  username text not null,
  username_key text generated always as (lower(username)) stored,
  display_name text not null,
  -- "preset:1" to "preset:12", or a photo from Google or Discord.
  avatar text not null default 'preset:1',
  username_changed_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),

  constraint profiles_username_format check (username ~ '^[A-Za-z0-9_]{3,20}$'),
  constraint profiles_username_unique unique (username_key),
  constraint profiles_display_name_len check (char_length(btrim(display_name)) between 1 and 40),
  constraint profiles_avatar_ok check (
    avatar ~ '^preset:([1-9]|1[0-2])$'
    or avatar ~ '^https://(lh3\.googleusercontent\.com|cdn\.discordapp\.com)/[^[:space:]]{1,400}$'
  )
);

-- Rules that a plain constraint can't express: reserved names, how often the
-- username can change, and fields nobody may rewrite.
create or replace function public.profiles_guard()
returns trigger
language plpgsql
set search_path = ''
as $$
begin
  if tg_op = 'INSERT' then
    if exists (select 1 from public.reserved_usernames r where r.name = lower(new.username)) then
      raise exception 'username_reserved' using errcode = 'P0001';
    end if;
    new.username_changed_at := null;
    new.created_at := now();
  else
    new.id := old.id;
    new.created_at := old.created_at;
    if lower(new.username) <> lower(old.username) then
      if exists (select 1 from public.reserved_usernames r where r.name = lower(new.username)) then
        raise exception 'username_reserved' using errcode = 'P0001';
      end if;
      -- The first change is free (a typo at sign-up); after that, once every 30 days.
      if old.username_changed_at is not null and old.username_changed_at > now() - interval '30 days' then
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

drop trigger if exists profiles_guard on public.profiles;
create trigger profiles_guard
  before insert or update on public.profiles
  for each row execute function public.profiles_guard();

-- ── Who can do what ────────────────────────────────────────────────────────

alter table public.profiles enable row level security;

drop policy if exists "profiles are visible to signed-in users" on public.profiles;
create policy "profiles are visible to signed-in users"
  on public.profiles for select to authenticated
  using (true);

drop policy if exists "people create their own profile" on public.profiles;
create policy "people create their own profile"
  on public.profiles for insert to authenticated
  with check ((select auth.uid()) = id);

drop policy if exists "people edit their own profile" on public.profiles;
create policy "people edit their own profile"
  on public.profiles for update to authenticated
  using ((select auth.uid()) = id)
  with check ((select auth.uid()) = id);

-- No delete policy: an account is deleted with delete_my_account() below.

-- ── Functions the app and site call ────────────────────────────────────────

-- Is this username free? Used while someone types it. Case does not matter.
create or replace function public.username_available(candidate text)
returns boolean
language sql
stable
security definer
set search_path = ''
as $$
  select candidate ~ '^[A-Za-z0-9_]{3,20}$'
    and not exists (select 1 from public.reserved_usernames r where r.name = lower(candidate))
    and not exists (select 1 from public.profiles p where p.username_key = lower(candidate));
$$;

revoke all on function public.username_available(text) from public;
grant execute on function public.username_available(text) to anon, authenticated;

-- Delete my account and everything that hangs off it (the profile goes with it).
create or replace function public.delete_my_account()
returns void
language plpgsql
security definer
set search_path = ''
as $$
begin
  if auth.uid() is null then
    raise exception 'not_signed_in';
  end if;
  delete from auth.users where id = auth.uid();
end;
$$;

revoke all on function public.delete_my_account() from public, anon;
grant execute on function public.delete_my_account() to authenticated;

-- Everything Velmora holds about me, as JSON. Grows as more features arrive.
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
    'profile', to_jsonb(p) - 'username_key'
  )
  from auth.users u
  left join public.profiles p on p.id = u.id
  where u.id = auth.uid();
$$;

revoke all on function public.export_my_data() from public, anon;
grant execute on function public.export_my_data() to authenticated;
