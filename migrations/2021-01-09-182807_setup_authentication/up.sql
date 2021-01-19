-- Your SQL goes here
create table if not exists users (
    id serial primary key,
    username text not null unique,
    email text not null unique,
    password text not null,
    created timestamp not null default now(),
    timezone text not null,
    email_verified boolean not null default 'f'
)
