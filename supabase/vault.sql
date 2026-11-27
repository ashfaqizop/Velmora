-- Velmora download keys: the backup of each install's vault key.
--
-- Run this in Supabase's SQL Editor AFTER schema.sql. Safe to run again.
--
-- Downloads are saved as .vlm files, encrypted with a key made on the device
-- (ChaCha20, see universeapi/src/vault.rs), so only Velmora plays them. If that
-- key is lost, say on a reinstall, every download is lost with it. So the key
-- is kept in the account too: one per platform (Android, Windows), readable and
-- writable only by its owner. Signing in on a new install brings the key back
-- and the old downloads play again.
--
-- The key is stored as sent (base64 of 32 bytes). Row-level security keeps it
-- to the account's owner; it is not encrypted again on top of that.

create table if not exists public.vault_keys (
  user_id uuid not null references auth.users (id) on delete cascade,
  platform text not null check (platform in ('android', 'windows')),
  key text not null check (char_length(key) = 44),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (user_id, platform)
);

alter table public.vault_keys enable row level security;

drop policy if exists "own vault key: read" on public.vault_keys;
create policy "own vault key: read" on public.vault_keys
  for select to authenticated using ((select auth.uid()) = user_id);

drop policy if exists "own vault key: add" on public.vault_keys;
create policy "own vault key: add" on public.vault_keys
  for insert to authenticated with check ((select auth.uid()) = user_id);

drop policy if exists "own vault key: change" on public.vault_keys;
create policy "own vault key: change" on public.vault_keys
  for update to authenticated using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);

drop policy if exists "own vault key: remove" on public.vault_keys;
create policy "own vault key: remove" on public.vault_keys
  for delete to authenticated using ((select auth.uid()) = user_id);

revoke all on public.vault_keys from anon;
grant select, insert, update, delete on public.vault_keys to authenticated;
