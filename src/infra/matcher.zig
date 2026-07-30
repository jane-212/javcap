const std = @import("std");
const Allocator = std.mem.Allocator;

fn toCodepoints(alloc: Allocator, s: []const u8) ![]u21 {
    var list = std.ArrayList(u21).init(alloc);
    errdefer list.deinit();

    var view = std.unicode.Utf8View.init(s) catch return error.InvalidUtf8;
    var it = view.iterator();
    while (it.nextCodepoint()) |cp| try list.append(cp);

    return try list.toOwnedSlice();
}

pub fn jaroWinkler(alloc: Allocator, a: []const u8, b: []const u8) !f64 {
    if (a.len == 0 and b.len == 0) return 1.0;
    if (a.len == 0 or b.len == 0) return 0.0;

    const ca = try toCodepoints(alloc, a);
    defer alloc.free(ca);

    const cb = try toCodepoints(alloc, b);
    defer alloc.free(cb);

    const na = ca.len;
    const nb = cb.len;

    if (na == 0 and nb == 0) return 1.0;
    if (na == 0 or nb == 0) return 0.0;

    const window = @max(@as(i64, @intCast(@max(na, nb))) / 2 - 1, 0);

    var arena = std.heap.ArenaAllocator.init(alloc);
    defer arena.deinit();
    const aa = arena.allocator();

    const matched_a = try aa.alloc(bool, na);
    const matched_b = try aa.alloc(bool, nb);
    @memset(matched_a, false);
    @memset(matched_b, false);

    var m: usize = 0;
    for (ca, 0..) |cp, i| {
        if (window == 0) {
            if (i < nb and !matched_b[i] and cp == cb[i]) {
                matched_a[i] = true;
                matched_b[i] = true;
                m += 1;
            }
        } else {
            const start = @max(0, @as(i64, @intCast(i)) - window);
            const end = @min(nb, @as(usize, @intCast(@as(i64, @intCast(i)) + window + 1)));
            var j = @as(usize, @intCast(start));
            while (j < end) : (j += 1) {
                if (!matched_b[j] and cp == cb[j]) {
                    matched_a[i] = true;
                    matched_b[j] = true;
                    m += 1;
                    break;
                }
            }
        }
    }

    if (m == 0) return 0.0;

    var t: usize = 0;
    var k: usize = 0;
    for (ca, 0..) |cp, i| {
        if (!matched_a[i]) continue;
        while (!matched_b[k]) k += 1;
        if (cp != cb[k]) t += 1;
        k += 1;
    }
    t /= 2;

    const fm = @as(f64, @floatFromInt(m));
    const fa = @as(f64, @floatFromInt(na));
    const fb = @as(f64, @floatFromInt(nb));
    const ft = @as(f64, @floatFromInt(t));
    const jaro = (fm / fa + fm / fb + (fm - ft) / fm) / 3.0;

    var prefix: usize = 0;
    const max_prefix = @min(@as(usize, 4), na, nb);
    while (prefix < max_prefix and ca[prefix] == cb[prefix]) prefix += 1;

    return jaro + @as(f64, @floatFromInt(prefix)) * 0.1 * (1.0 - jaro);
}
