/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

create table if not exists "class" (
    id serial primary key,
    name text not null,
    description text not null,
    created timestamp not null,
    code text not null unique
);

create table if not exists class_teacher (
    id serial primary key,
    user_id integer not null references users (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade
);

create table if not exists class_student (
    id serial primary key,
    user_id integer not null references users (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade
);

create table if not exists class_teacher_invite (
    id serial primary key,
    inviting_user_id integer not null references users (id) on delete cascade,
    invited_user_id integer not null references users (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade,
    accepted boolean not null
)
