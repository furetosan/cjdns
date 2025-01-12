/* vim: set expandtab ts=4 sw=4: */
/*
 * You may redistribute this program and/or modify it under the terms of
 * the GNU General Public License as published by the Free Software Foundation,
 * either version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
#include "crypto/random/Random.h"
#include "memory/Allocator.h"

#include <unistd.h>

int randombytes_main(void);
int randombytes_main()
{
    struct Allocator* alloc = Allocator_new(1<<20);
    struct Random* rand = NULL;
    Err_assert(Random_new(&rand, alloc, NULL));

    struct {
        uint64_t align;
        uint8_t buff[4096];
    } str;

    size_t out = 0;
    for (;;) {
        Random_bytes(rand, str.buff, 4096);
        out = write(STDOUT_FILENO, str.buff, 4096);
    }
    return (out == 4096) ? 0 : -1;
}
