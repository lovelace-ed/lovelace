# License

These licenses set out the terms under which you may use this code, and how any code you submit to
the project is licensed.

## Present licenses

Different parts of this codebase are licensed differently:
- Malvolio (code contained in `utils/malvolio`) is licensed according to the terms of the Mozilla
Public License v2.0.
- Prospero (code contained in `utils/prospero`) is licensed according to the terms of the MIT
License *OR* the terms of the Apache License 2.0 – at your option.
- All other code is licensed under the GNU Affero General Public License
- Standalone documentation (what little of it that there is) – i.e. documentation which is not
included in comments inside source code files – is licensed under the terms of the GNU Free
Documentation License.

A copy of all of these licenses can be found in the `licences` directory (at the root of this
project). In opening a pull request to this repository, you agree that your contributions will be
licensed according to the license terms.

## Licensing code (guidelines)

**Important:** these are just guidelines – if you think something should be licensed differently,
that's fine and we'd love to discuss :D

If you're adding new code to the project, use this rule of thumb for licensing:
* If you're adding code which is part of the Lovelace application, use the AGPL-3.0
* If you're adding code which is a free-standing library (which is used inside the application),
use the MPL-2.0, *unless* the library is heavily linked to other libraries with different licenses
(e.g. if it exposes types from an MIT-licensed project in its public API it should probably be
licensed under the MIT-license)
* If you're writing documentation (e.g. user manuals, contribution guidelines, etc), use the FDL-1.3
* If in doubt, don't hesitate to ask! (you can
[ask here](https://github.com/lovelace-ed/lovelace/discussions)).
