create table if not exists progress (
	progress_id text primary key default generate_prefixed_ksuid('progress_id'),
	movie_index bigint not null,
	created_at timestamptz not null default current_timestamp,
	updated_at timestamptz not null default current_timestamp
);
select trigger_updated_at('progress');
