-- Velmora Social, phase 3: activity, ratings and reviews, lists, the feed,
-- stats, suggestions and leaderboards.
--
-- Run in Supabase's SQL Editor AFTER schema.sql, sync.sql and social.sql.
-- Safe to run again.
--
-- Titles are named by TMDB ("movie/603", "tv/1396"). Names and posters are
-- not stored here: the website looks them up from TMDB, so nobody can rename
-- a film by posting about it. 18+ titles are never posted: the app leaves them
-- out, and they have no TMDB number anyway.

-- ── Shared helpers ─────────────────────────────────────────────────────────

create or replace function public.valid_title_key(k text)
returns boolean
language sql
immutable
set search_path = ''
as $$ select coalesce(k, '') ~ '^(movie|tv)/[0-9]{1,9}$' $$;

-- Can people see this person's things? (Not hidden, not banned.)
create or replace function public.profile_visible(uid uuid)
returns boolean
language sql
stable
security definer
set search_path = ''
as $$ select exists (select 1 from public.profiles p where p.id = uid and not p.hidden and not p.banned) $$;

-- Is this person sharing what they watch?
create or replace function public.profile_sharing(uid uuid)
returns boolean
language sql
stable
security definer
set search_path = ''
as $$ select exists (select 1 from public.profiles p where p.id = uid and p.share_activity and not p.hidden and not p.banned) $$;

-- Lists of titles: [{ "k": "movie/603", "name": "...", "year": 1999, "poster": "https://image.tmdb.org/..." }]
create or replace function public.valid_title_items(items jsonb, most integer)
returns boolean
language plpgsql
immutable
set search_path = ''
as $$
declare
  item jsonb;
begin
  if jsonb_typeof(items) <> 'array' or jsonb_array_length(items) > most then return false; end if;
  for item in select value from jsonb_array_elements(items) loop
    if jsonb_typeof(item) <> 'object' then return false; end if;
    if not public.valid_title_key(item ->> 'k') then return false; end if;
    if char_length(coalesce(item ->> 'name', '')) not between 1 and 200 then return false; end if;
    if item ? 'year' and jsonb_typeof(item -> 'year') not in ('number', 'null') then return false; end if;
    if item ? 'poster' and jsonb_typeof(item -> 'poster') <> 'null'
       and coalesce(item ->> 'poster', '') !~ '^https://image\.tmdb\.org/t/p/[a-z0-9]{2,10}/[A-Za-z0-9_.-]{1,80}$' then
      return false;
    end if;
  end loop;
  return true;
end;
$$;

-- ── Activity ───────────────────────────────────────────────────────────────

create table if not exists public.activity (
  id bigint generated always as identity primary key,
  user_id uuid not null references public.profiles (id) on delete cascade,
  kind text not null check (kind in ('watching', 'finished', 'listed', 'rated')),
  k text not null check (public.valid_title_key(k)),
  season integer check (season between 0 and 999),
  episode integer check (episode between 0 and 9999),
  -- Minutes watched (finished), summed when several episodes merge into one row.
  minutes integer not null default 0 check (minutes between 0 and 1000000),
  -- How many episodes or plays this row stands for.
  plays integer not null default 1 check (plays between 1 and 100000),
  score smallint check (score between 1 and 10),
  created_at timestamptz not null default now()
);
create index if not exists activity_user on public.activity (user_id, created_at desc);
create index if not exists activity_recent on public.activity (created_at desc);
create index if not exists activity_title on public.activity (k, created_at desc);

alter table public.activity enable row level security;
drop policy if exists "shared activity is public" on public.activity;
create policy "shared activity is public" on public.activity for select to anon, authenticated
  using (user_id = (select auth.uid()) or public.profile_sharing(user_id));
drop policy if exists "remove own activity" on public.activity;
create policy "remove own activity" on public.activity for delete to authenticated
  using (user_id = (select auth.uid()));
-- No insert or update policy: rows come only through post_activity below.

-- Post something from the app. Quietly does nothing when sharing is off.
-- Episodes of the same show within 12 hours fold into one row, so the feed
-- says "finished 4 episodes of X" instead of four lines.
create or replace function public.post_activity(
  what text,
  title text,
  season_no integer default null,
  episode_no integer default null,
  watched_minutes integer default 0
)
returns void
language plpgsql
security definer
set search_path = ''
as $$
declare
  me uuid := auth.uid();
  row_id bigint;
  mins integer := least(greatest(coalesce(watched_minutes, 0), 0), 600);
begin
  if me is null or what not in ('watching', 'finished', 'listed') or not public.valid_title_key(title) then return; end if;
  if not public.profile_sharing(me) then return; end if;
  if (select count(*) from public.activity a where a.user_id = me and a.created_at > now() - interval '1 day') >= 400 then return; end if;

  if what = 'listed' then
    if exists (select 1 from public.activity a where a.user_id = me and a.k = title and a.kind = 'listed' and a.created_at > now() - interval '30 days') then
      return;
    end if;
    insert into public.activity (user_id, kind, k) values (me, 'listed', title);
    return;
  end if;

  if what = 'finished' then
    -- "Watching" gives way to "finished" for the same title.
    delete from public.activity a where a.user_id = me and a.k = title and a.kind = 'watching' and a.created_at > now() - interval '12 hours';
  elsif exists (select 1 from public.activity a where a.user_id = me and a.k = title and a.kind = 'finished' and a.created_at > now() - interval '12 hours') then
    -- Already finishing episodes of it today: no need to also say "watching".
    return;
  end if;

  select a.id into row_id from public.activity a
  where a.user_id = me and a.k = title and a.kind = what and a.created_at > now() - interval '12 hours'
  order by a.created_at desc limit 1;

  if row_id is null then
    insert into public.activity (user_id, kind, k, season, episode, minutes)
    values (me, what, title, season_no, episode_no, case when what = 'finished' then mins else 0 end);
  else
    update public.activity a set
      season = coalesce(season_no, a.season),
      episode = coalesce(episode_no, a.episode),
      minutes = least(a.minutes + case when what = 'finished' then mins else 0 end, 1000000),
      plays = least(a.plays + case when what = 'finished' then 1 else 0 end, 100000),
      created_at = now()
    where a.id = row_id;
  end if;
end;
$$;
revoke all on function public.post_activity(text, text, integer, integer, integer) from public;
grant execute on function public.post_activity(text, text, integer, integer, integer) to authenticated;

-- ── Ratings and reviews ────────────────────────────────────────────────────

create table if not exists public.ratings (
  user_id uuid not null references public.profiles (id) on delete cascade,
  k text not null check (public.valid_title_key(k)),
  -- 1 to 10, shown as half stars out of five.
  score smallint not null check (score between 1 and 10),
  review text not null default '' check (char_length(review) <= 500),
  spoiler boolean not null default false,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (user_id, k)
);
create index if not exists ratings_title on public.ratings (k, updated_at desc);
create index if not exists ratings_recent on public.ratings (updated_at desc);

alter table public.ratings enable row level security;
drop policy if exists "ratings are public" on public.ratings;
create policy "ratings are public" on public.ratings for select to anon, authenticated
  using (user_id = (select auth.uid()) or public.profile_visible(user_id));
drop policy if exists "rate as yourself" on public.ratings;
create policy "rate as yourself" on public.ratings for insert to authenticated with check (user_id = (select auth.uid()));
drop policy if exists "edit own rating" on public.ratings;
create policy "edit own rating" on public.ratings for update to authenticated
  using (user_id = (select auth.uid())) with check (user_id = (select auth.uid()));
drop policy if exists "remove own rating" on public.ratings;
create policy "remove own rating" on public.ratings for delete to authenticated using (user_id = (select auth.uid()));

create or replace function public.ratings_guard()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if tg_op = 'INSERT' then
    if (select count(*) from public.ratings r where r.user_id = new.user_id and r.created_at > now() - interval '1 day') >= 100 then
      raise exception 'rating_limit' using errcode = 'P0001';
    end if;
    new.created_at := now();
  else
    new.user_id := old.user_id;
    new.k := old.k;
    new.created_at := old.created_at;
  end if;
  new.review := btrim(new.review);
  new.updated_at := now();
  return new;
end;
$$;
drop trigger if exists ratings_guard on public.ratings;
create trigger ratings_guard before insert or update on public.ratings
  for each row execute function public.ratings_guard();

-- A rating shows in the feed (once a day per title) when sharing is on.
create or replace function public.ratings_activity()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if tg_op = 'DELETE' then
    delete from public.activity a where a.user_id = old.user_id and a.k = old.k and a.kind = 'rated';
    return null;
  end if;
  if not public.profile_sharing(new.user_id) then return null; end if;
  update public.activity a set score = new.score, created_at = now()
  where a.user_id = new.user_id and a.k = new.k and a.kind = 'rated' and a.created_at > now() - interval '1 day';
  if not found then
    insert into public.activity (user_id, kind, k, score) values (new.user_id, 'rated', new.k, new.score);
  end if;
  return null;
end;
$$;
drop trigger if exists ratings_activity on public.ratings;
create trigger ratings_activity after insert or update or delete on public.ratings
  for each row execute function public.ratings_activity();

-- The Velmora score for a title: average out of 10 and how many rated it.
create or replace function public.title_score(title text)
returns table (average numeric, ratings integer, reviews integer)
language sql
stable
set search_path = ''
as $$
  select round(avg(r.score)::numeric, 1), count(*)::int, count(*) filter (where r.review <> '')::int
  from public.ratings r
  where r.k = title and public.profile_visible(r.user_id);
$$;
grant execute on function public.title_score(text) to anon, authenticated;

-- Best rated titles with at least `least_ratings` ratings.
create or replace function public.top_rated(lim integer default 30, least_ratings integer default 3)
returns table (k text, average numeric, ratings integer)
language sql
stable
set search_path = ''
as $$
  select r.k, round(avg(r.score)::numeric, 1), count(*)::int
  from public.ratings r
  where public.profile_visible(r.user_id)
  group by r.k
  having count(*) >= greatest(coalesce(least_ratings, 3), 1)
  order by avg(r.score) desc, count(*) desc
  limit least(greatest(coalesce(lim, 30), 1), 60);
$$;
grant execute on function public.top_rated(integer, integer) to anon, authenticated;

-- ── Lists ──────────────────────────────────────────────────────────────────

create table if not exists public.lists (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references public.profiles (id) on delete cascade,
  name text not null check (char_length(btrim(name)) between 1 and 60),
  description text not null default '' check (char_length(description) <= 300),
  items jsonb not null default '[]'::jsonb check (public.valid_title_items(items, 100)),
  public boolean not null default true,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
create index if not exists lists_user on public.lists (user_id, updated_at desc);

alter table public.lists enable row level security;
drop policy if exists "public lists" on public.lists;
create policy "public lists" on public.lists for select to anon, authenticated
  using (user_id = (select auth.uid()) or (public and public.profile_visible(user_id)));
drop policy if exists "make own lists" on public.lists;
create policy "make own lists" on public.lists for insert to authenticated with check (user_id = (select auth.uid()));
drop policy if exists "edit own lists" on public.lists;
create policy "edit own lists" on public.lists for update to authenticated
  using (user_id = (select auth.uid())) with check (user_id = (select auth.uid()));
drop policy if exists "remove own lists" on public.lists;
create policy "remove own lists" on public.lists for delete to authenticated using (user_id = (select auth.uid()));

create or replace function public.lists_guard()
returns trigger
language plpgsql
security definer
set search_path = ''
as $$
begin
  if tg_op = 'INSERT' then
    if (select count(*) from public.lists l where l.user_id = new.user_id) >= 50 then
      raise exception 'list_limit' using errcode = 'P0001';
    end if;
    new.created_at := now();
  else
    new.user_id := old.user_id;
    new.created_at := old.created_at;
  end if;
  new.name := btrim(new.name);
  new.description := btrim(new.description);
  new.updated_at := now();
  return new;
end;
$$;
drop trigger if exists lists_guard on public.lists;
create trigger lists_guard before insert or update on public.lists
  for each row execute function public.lists_guard();

-- ── The feed: what the people you follow (and you) did lately ──────────────

create or replace function public.my_feed(before timestamptz default null, lim integer default 30)
returns table (
  id bigint,
  kind text,
  k text,
  season integer,
  episode integer,
  minutes integer,
  plays integer,
  score smallint,
  created_at timestamptz,
  review text,
  spoiler boolean,
  user_id uuid,
  username text,
  display_name text,
  avatar text,
  verified boolean
)
language sql
stable
set search_path = ''
as $$
  select a.id, a.kind, a.k, a.season, a.episode, a.minutes, a.plays, a.score, a.created_at,
         case when a.kind = 'rated' then r.review end, coalesce(r.spoiler, false),
         p.id, p.username, p.display_name, p.avatar, p.verified
  from public.activity a
  join public.profiles p on p.id = a.user_id
  left join public.ratings r on a.kind = 'rated' and r.user_id = a.user_id and r.k = a.k
  where (a.user_id = auth.uid() or a.user_id in (select f.followee from public.follows f where f.follower = auth.uid()))
    and a.created_at < coalesce(before, now() + interval '1 minute')
  order by a.created_at desc
  limit least(greatest(coalesce(lim, 30), 1), 60);
$$;
revoke all on function public.my_feed(timestamptz, integer) from public;
grant execute on function public.my_feed(timestamptz, integer) to authenticated;

-- One person's activity, for their profile.
create or replace function public.user_activity(target uuid, before timestamptz default null, lim integer default 30)
returns table (
  id bigint,
  kind text,
  k text,
  season integer,
  episode integer,
  minutes integer,
  plays integer,
  score smallint,
  created_at timestamptz,
  review text,
  spoiler boolean
)
language sql
stable
set search_path = ''
as $$
  select a.id, a.kind, a.k, a.season, a.episode, a.minutes, a.plays, a.score, a.created_at,
         case when a.kind = 'rated' then r.review end, coalesce(r.spoiler, false)
  from public.activity a
  left join public.ratings r on a.kind = 'rated' and r.user_id = a.user_id and r.k = a.k
  where a.user_id = target and a.created_at < coalesce(before, now() + interval '1 minute')
  order by a.created_at desc
  limit least(greatest(coalesce(lim, 30), 1), 60);
$$;
grant execute on function public.user_activity(uuid, timestamptz, integer) to anon, authenticated;

-- ── Stats for a profile (from what they share) ─────────────────────────────

create or replace function public.profile_stats(target uuid)
returns jsonb
language sql
stable
security definer
set search_path = ''
as $$
  select case when target = auth.uid() or public.profile_sharing(target) then jsonb_build_object(
    'minutes', coalesce((select sum(a.minutes) from public.activity a where a.user_id = target and a.kind = 'finished'), 0),
    'minutes_month', coalesce((select sum(a.minutes) from public.activity a where a.user_id = target and a.kind = 'finished' and a.created_at > now() - interval '30 days'), 0),
    'movies', (select count(distinct a.k) from public.activity a where a.user_id = target and a.kind = 'finished' and a.k like 'movie/%'),
    'shows', (select count(distinct a.k) from public.activity a where a.user_id = target and a.kind = 'finished' and a.k like 'tv/%'),
    'episodes', coalesce((select sum(a.plays) from public.activity a where a.user_id = target and a.kind = 'finished' and a.k like 'tv/%'), 0),
    'ratings', (select count(*) from public.ratings r where r.user_id = target),
    'reviews', (select count(*) from public.ratings r where r.user_id = target and r.review <> ''),
    'average', (select round(avg(r.score)::numeric, 1) from public.ratings r where r.user_id = target),
    -- The titles they spent most time on, for "top genres" (looked up on TMDB by the website).
    'top', coalesce((select jsonb_agg(t.k) from (
      select a.k from public.activity a where a.user_id = target and a.kind = 'finished'
      group by a.k order by sum(a.minutes) desc limit 40) t), '[]'::jsonb)
  ) else null end;
$$;
grant execute on function public.profile_stats(uuid) to anon, authenticated;

-- ── People you might know: followed by people you follow ──────────────────

create or replace function public.suggested_people(lim integer default 12)
returns table (
  id uuid,
  username text,
  display_name text,
  avatar text,
  bio text,
  verified boolean,
  followers_count integer,
  mutual integer
)
language sql
stable
security definer
set search_path = ''
as $$
  with mine as (select f.followee from public.follows f where f.follower = auth.uid())
  select p.id, p.username, p.display_name, p.avatar, p.bio, p.verified, p.followers_count, count(*)::int
  from public.follows f
  join mine m on f.follower = m.followee
  join public.profiles p on p.id = f.followee
  where auth.uid() is not null
    and f.followee <> auth.uid()
    and f.followee not in (select followee from mine)
    and not p.hidden and not p.banned
  group by p.id
  order by count(*) desc, p.followers_count desc
  limit least(greatest(coalesce(lim, 12), 1), 30);
$$;
revoke all on function public.suggested_people(integer) from public;
grant execute on function public.suggested_people(integer) to authenticated;

-- ── Leaderboards (last 30 days) ────────────────────────────────────────────
--   'hours'     most time watched (sharing profiles only)
--   'reviews'   most reviews written
--   'followers' most followed (all time)

create or replace function public.leaderboard(board text, lim integer default 20)
returns table (
  id uuid,
  username text,
  display_name text,
  avatar text,
  verified boolean,
  followers_count integer,
  value integer
)
language sql
stable
security definer
set search_path = ''
as $$
  select p.id, p.username, p.display_name, p.avatar, p.verified, p.followers_count, t.value
  from (
    select a.user_id as uid, (sum(a.minutes) / 60)::int as value
    from public.activity a
    where board = 'hours' and a.kind = 'finished' and a.created_at > now() - interval '30 days'
    group by a.user_id
    union all
    select r.user_id, count(*)::int
    from public.ratings r
    where board = 'reviews' and r.review <> '' and r.updated_at > now() - interval '30 days'
    group by r.user_id
    union all
    select p2.id, p2.followers_count
    from public.profiles p2
    where board = 'followers' and p2.followers_count > 0
  ) t
  join public.profiles p on p.id = t.uid
  where not p.hidden and not p.banned
    and (board <> 'hours' or p.share_activity)
    and t.value > 0
  order by t.value desc, p.followers_count desc
  limit least(greatest(coalesce(lim, 20), 1), 50);
$$;
grant execute on function public.leaderboard(text, integer) to anon, authenticated;

-- ── Reports can point at a review ──────────────────────────────────────────

alter table public.reports add column if not exists title_key text check (title_key is null or public.valid_title_key(title_key));

-- ── Export includes all of it ──────────────────────────────────────────────

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
    'settings', (select s.data from public.user_settings s where s.user_id = u.id),
    'ratings', coalesce((select jsonb_agg(to_jsonb(r) - 'user_id' order by r.updated_at desc) from public.ratings r where r.user_id = u.id), '[]'::jsonb),
    'lists', coalesce((select jsonb_agg(to_jsonb(l) - 'user_id' order by l.updated_at desc) from public.lists l where l.user_id = u.id), '[]'::jsonb),
    'activity', coalesce((select jsonb_agg(to_jsonb(a) - 'user_id' order by a.created_at desc) from public.activity a where a.user_id = u.id), '[]'::jsonb)
  )
  from auth.users u
  left join public.profiles p on p.id = u.id
  where u.id = auth.uid();
$$;
