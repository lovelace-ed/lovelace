/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

//! Constants mapping HTML structures to the CSS classes we use to make them
//! display properly in browsers.
//!
//! If you add constants to this list (please do if the need arises!) please add
//! them under the relevant heading.

/* Lists */
pub const LIST: &str = "list";
pub const LIST_ITEM: &str = "list-item";
/* Forms */
/// Normally applied to a div containing an <input> or <select> element.
pub const FORM_GROUP: &str = "form-group";
