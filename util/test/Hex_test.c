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
#include "util/Bits.h"
#include "util/Hex.h"
#include "util/Assert.h"

#include <stdio.h>

int main()
{
    struct Allocator* alloc = Allocator_new(20000);
    struct Random* rand = NULL;
    Err_assert(Random_new(&rand, alloc, NULL));

    uint8_t bytes[32];
    Random_bytes(rand, bytes, 32);

    uint8_t hex[65] = {0};

    Assert_true(Hex_encode(hex, 65, bytes, 32) == 64);

    //printf("hex encoded: %s\n", hex);

    uint8_t bytes2[32];
    Assert_true(Hex_decode(bytes2, 32, hex, 64) == 32);

    Assert_true(Bits_memcmp(bytes, bytes2, 32) == 0);

    Allocator_free(alloc);
    return 0;
}
