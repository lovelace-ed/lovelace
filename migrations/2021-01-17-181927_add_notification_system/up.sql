/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
create table if not exists notifications (
    id serial primary key,
    title text not null,
    contents text not null,
    created_at timestamp not null default now(),
    /* should go from something like
        0 => "info" (i.e. not really important)
        1 => "warning" (i.e. person should take note of this)
        2 => "danger" (i.e. person must take note of this)
    */
    priority smallint not null,
    user_id integer not null references users (id) on delete cascade,
    read boolean not null
)
