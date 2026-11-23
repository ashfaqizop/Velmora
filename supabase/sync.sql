-- Velmora sync: My List, watch progress and settings, kept per account.
--
-- Run this in Supabase's SQL Editor AFTER schema.sql. Safe to run again.
--
-- The app keeps everything on the PC first and pushes changes in the
-- background, so it is instant and works offline. Each row carries the time of
-- the edit on the PC that made it (updated_at): the newest edit wins. synced_at
-- is the server's own clock, which is what other PCs page through to find what
-- is new. Removals are kept as small "deleted" rows so they reach other PCs too.

-- ── My List ────────────────────────────────────────────────────────────────

create table if not exists public.list_items (
  user_id uuid not null references auth.users (id) on delete cascade,
  title_key text not null check (char_length(title_key) between 1 and 200),
  title jsonb,
  updated_at timestamptz not null,
  deleted boolean not null default false,
  synced_at timestamptz not null default now(),
  primary key (user_id, title_key),
  constraint list_items_size check (title is null or octet_length(title::text) <= 20000)
);
create index if not exists list_items_synced on public.list_items (user_id, synced_at);

-- ── Watch progress and history (one row per title) ─────────────────────────

create table if not exists public.progress (
  user_id uuid not null references auth.users (id) on delete cascade,
  title_key text not null check (char_length(title_key) between 1 and 200),
  entry jsonb,
  updated_at timestamptz not null,
  deleted boolean not null default false,
  synced_at timestamptz not null default now(),
  primary key (user_id, title_key),
  constraint progress_size check (entry is null or octet_length(entry::text) <= 24000)
);
create index if not exists progress_synced on public.progress (user_id, synced_at);

-- ── Settings (one row per account) ─────────────────────────────────────────

create table if not exists public.user_settings (
  user_id uuid primary key references auth.users (id) on delete cascade,
  data jsonb not null default '{}'::jsonb,
  updated_at timestamptz not null,
  synced_at timestamptz not null default now(),
  constraint user_settings_size check (octet_length(data::text) <= 20000)
);

-- ── Who can do what: your own rows, nothing else ───────────────────────────

alter table public.list_items enable row level security;
alter table public.progress enable row level security;
alter table public.user_settings enable row level security;

drop policy if exists "own list rows" on public.list_items;
create policy "own list rows" on public.list_items for select to authenticated using ((select auth.uid()) = user_id);
drop policy if exists "add own list rows" on public.list_items;
create policy "add own list rows" on public.list_items for insert to authenticated with check ((select auth.uid()) = user_id);
drop policy if exists "edit own list rows" on public.list_items;
create policy "edit own list rows" on public.list_items for update to authenticated
  using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);

drop policy if exists "own progress rows" on public.progress;
create policy "own progress rows" on public.progress for select to authenticated using ((select auth.uid()) = user_id);
drop policy if exists "add own progress rows" on public.progress;
create policy "add own progress rows" on public.progress for insert to authenticated with check ((select auth.uid()) = user_id);
drop policy if exists "edit own progress rows" on public.progress;
create policy "edit own progress rows" on public.progress for update to authenticated
  using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);

drop policy if exists "own settings" on public.user_settings;
create policy "own settings" on public.user_settings for select to authenticated using ((select auth.uid()) = user_id);
drop policy if exists "add own settings" on public.user_settings;
create policy "add own settings" on public.user_settings for insert to authenticated with check ((select auth.uid()) = user_id);
drop policy if exists "edit own settings" on public.user_settings;
create policy "edit own settings" on public.user_settings for update to authenticated
  using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);

-- No delete policies: removals are "deleted" rows, and deleting the account
-- removes everything through the foreign keys.

-- ── Pushing changes ────────────────────────────────────────────────────────
-- Several rows at once; a row only replaces what is there if its edit is newer.

create or replace function public.sync_push_list(rows jsonb)
returns void
language plpgsql
set search_path = ''
as $$
begin
  if auth.uid() is null then raise exception 'not_signed_in'; end if;
  if jsonb_typeof(rows) <> 'array' or jsonb_array_length(rows) > 500 then raise exception 'too_many_rows'; end if;

  insert into public.list_items (user_id, title_key, title, updated_at, deleted, synced_at)
  select auth.uid(), r.title_key, case when r.deleted then null else r.title end, r.updated_at, coalesce(r.deleted, false), now()
  from jsonb_to_recordset(rows) as r(title_key text, title jsonb, updated_at timestamptz, deleted boolean)
  on conflict (user_id, title_key) do update
    set title = excluded.title,
        updated_at = excluded.updated_at,
        deleted = excluded.deleted,
        synced_at = now()
    where excluded.updated_at > public.list_items.updated_at;
end;
$$;

create or replace function public.sync_push_progress(rows jsonb)
returns void
language plpgsql
set search_path = ''
as $$
begin
  if auth.uid() is null then raise exception 'not_signed_in'; end if;
  if jsonb_typeof(rows) <> 'array' or jsonb_array_length(rows) > 500 then raise exception 'too_many_rows'; end if;

  insert into public.progress (user_id, title_key, entry, updated_at, deleted, synced_at)
  select auth.uid(), r.title_key, case when r.deleted then null else r.entry end, r.updated_at, coalesce(r.deleted, false), now()
  from jsonb_to_recordset(rows) as r(title_key text, entry jsonb, updated_at timestamptz, deleted boolean)
  on conflict (user_id, title_key) do update
    set entry = excluded.entry,
        updated_at = excluded.updated_at,
        deleted = excluded.deleted,
        synced_at = now()
    where excluded.updated_at > public.progress.updated_at;
end;
$$;

create or replace function public.sync_push_settings(new_data jsonb, edited_at timestamptz)
returns void
language plpgsql
set search_path = ''
as $$
begin
  if auth.uid() is null then raise exception 'not_signed_in'; end if;

  insert into public.user_settings (user_id, data, updated_at, synced_at)
  values (auth.uid(), new_data, edited_at, now())
  on conflict (user_id) do update
    set data = excluded.data, updated_at = excluded.updated_at, synced_at = now()
    where excluded.updated_at > public.user_settings.updated_at;
end;
$$;

revoke all on function public.sync_push_list(jsonb) from public, anon;
revoke all on function public.sync_push_progress(jsonb) from public, anon;
revoke all on function public.sync_push_settings(jsonb, timestamptz) from public, anon;
grant execute on function public.sync_push_list(jsonb) to authenticated;
grant execute on function public.sync_push_progress(jsonb) to authenticated;
grant execute on function public.sync_push_settings(jsonb, timestamptz) to authenticated;

-- ── Exporting my data now includes what is synced ──────────────────────────

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

revoke all on function public.export_my_data() from public, anon;
grant execute on function public.export_my_data() to authenticated;
