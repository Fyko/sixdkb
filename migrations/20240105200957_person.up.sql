create table if not exists person (
	person_id text primary key default generate_prefixed_ksuid('person'),
	tmdb_id integer unique not null,
	name text not null,
	profile_path text,
	popularity real not null,
	created_at timestamptz not null default current_timestamp,
	updated_at timestamptz not null default current_timestamp
);
select trigger_updated_at('person');

create index on person(tmdb_id);
