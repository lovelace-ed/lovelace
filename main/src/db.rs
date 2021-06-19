#![allow(missing_debug_implementations)]
/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use diesel::connection::{AnsiTransactionManager, SimpleConnection};
use diesel::deserialize::QueryableByName;
use diesel::pg::Pg;
use diesel::query_builder::{AsQuery, QueryFragment, QueryId};
use diesel::r2d2::CustomizeConnection;
use diesel::sql_types::HasSqlType;
use diesel::{Connection, ConnectionResult, PgConnection, QueryResult, Queryable};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::{diesel, Config, PoolResult, Poolable};

embed_migrations!("../migrations/");

#[cfg(test)]
pub type DatabaseConnection = TestPgConnection;

#[cfg(not(test))]
pub type DatabaseConnection = PgConnection;

#[cfg(not(test))]
#[database("postgres")]
#[derive(Clone)]
pub struct Database(diesel::PgConnection);

#[derive(Debug)]
struct TestTransaction;

impl CustomizeConnection<TestPgConnection, diesel::r2d2::Error> for TestTransaction {
    fn on_acquire(&self, conn: &mut TestPgConnection) -> Result<(), diesel::r2d2::Error> {
        conn.begin_test_transaction().unwrap();
        Ok(())
    }
}

/// A connection useful for testing. All transactions are rolled back and are never committed to the
/// database. This should be used only for testing and not for any other purpose.
pub struct TestPgConnection(diesel::PgConnection);

impl SimpleConnection for TestPgConnection {
    fn batch_execute(&self, query: &str) -> QueryResult<()> {
        self.0.batch_execute(query)
    }
}

impl Connection for TestPgConnection {
    type Backend = Pg;
    type TransactionManager = AnsiTransactionManager;

    fn establish(database_url: &str) -> ConnectionResult<Self> {
        Ok(Self(PgConnection::establish(database_url)?))
    }

    fn execute(&self, query: &str) -> QueryResult<usize> {
        self.0.execute(query)
    }

    fn query_by_index<T, U>(&self, source: T) -> QueryResult<Vec<U>>
    where
        T: AsQuery,
        T::Query: QueryFragment<Pg> + QueryId,
        Pg: HasSqlType<T::SqlType>,
        U: Queryable<T::SqlType, Pg>,
    {
        self.0.query_by_index(source)
    }

    fn query_by_name<T, U>(&self, source: &T) -> QueryResult<Vec<U>>
    where
        T: QueryFragment<Pg> + QueryId,
        U: QueryableByName<Pg>,
    {
        self.0.query_by_name(source)
    }

    fn execute_returning_count<T>(&self, source: &T) -> QueryResult<usize>
    where
        T: QueryFragment<Pg> + QueryId,
    {
        self.0.execute_returning_count(source)
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        self.0.transaction_manager()
    }
}

impl Poolable for TestPgConnection {
    type Manager = diesel::r2d2::ConnectionManager<TestPgConnection>;
    type Error = rocket_sync_db_pools::r2d2::Error;

    fn pool(name: &str, rocket: &Rocket<Build>) -> PoolResult<Self> {
        let config = Config::from(name, rocket)?;
        let manager = diesel::r2d2::ConnectionManager::new(config.url);
        Ok(diesel::r2d2::Pool::builder()
            .connection_customizer(Box::new(TestTransaction))
            .max_size(config.pool_size)
            .build(manager)?)
    }
}

#[cfg(test)]
#[database("postgres")]
pub struct Database(TestPgConnection);

pub async fn run_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let conn = Database::get_one(&rocket)
        .await
        .expect("Couldn't create a database connection.");
    conn.run(|c| match embedded_migrations::run(c) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    })
    .await
}
