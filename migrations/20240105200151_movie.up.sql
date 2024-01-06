create table if not exists movie (
	movie_id text primary key default generate_prefixed_ksuid('movie'),
	tmdb_id integer unique not null,
	imdb_id text,
	title text not null,
	backdrop_path text,
	poster_path text,
	popularity real not null,
	created_at timestamptz not null default current_timestamp,
	updated_at timestamptz not null default current_timestamp
);
select trigger_updated_at('movie');

create index on movie(tmdb_id);
