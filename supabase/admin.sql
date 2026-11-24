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
