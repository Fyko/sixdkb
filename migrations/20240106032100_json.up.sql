-- Add up migration script here
create table if not exists movie_credits (
	id int primary key,
	data jsonb
);

create table if not exists movie_details (
	id int primary key,
	data jsonb
);
