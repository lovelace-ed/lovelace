/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

#[macro_export]
/// Attempts to retrieve a provided query from the database or returns a
/// database error message if that fails.
macro_rules! catch_database_error {
    ($exp:expr) => {
        match $exp {
            Ok(t) => t,
            Err(e) => {
                error!("{:#?}", e);
                return $crate::utils::error_messages::database_error();
            }
        }
    };
}
