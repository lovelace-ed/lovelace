/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

create table if not exists users (
    id serial primary key,
    username text not null unique,
    email text not null unique,
    password text not null,
    created timestamp not null default now(),
    timezone text not null,
    email_verified boolean not null default 'f'
)
