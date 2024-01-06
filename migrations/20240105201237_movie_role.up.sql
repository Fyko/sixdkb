create table if not exists movie_role (
	role_id text primary key default generate_prefixed_ksuid('role'),
	person_id text not null references person(person_id),
	movie_id text not null references movie(movie_id),
	character_or_job text not null,
	created_at timestamptz not null default current_timestamp,
	updated_at timestamptz not null default current_timestamp
);
select trigger_updated_at('movie_role');
