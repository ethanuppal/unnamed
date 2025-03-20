// Copyright (C) 2024 Ethan Uppal.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program.  If not, see <https://www.gnu.org/licenses/>.

let disablePrint = false

func print(_ items: Any..., separator: String = " ", terminator: String = "\n")
{
    if disablePrint {
        return
    }
    for i in items.indices {
        if i > 0 {
            Swift.print(separator, separator: "", terminator: "")
        }
        Swift.print(items[i], separator: "", terminator: "")
    }
    Swift.print(terminator, separator: "", terminator: "")
}
