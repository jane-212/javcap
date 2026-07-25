const std = @import("std");
const Allocator = std.mem.Allocator;
const domain = @import("domain");
const mecha = @import("mecha");

const Self = @This();

pub const ParsedFile = struct {
    alloc: Allocator,
    key: domain.jav.Key,
    name: []const u8,

    pub fn deinit(self: *ParsedFile) void {
        self.alloc.free(self.name);
        self.key.deinit(self.alloc);
        self.* = undefined;
    }
};

pub fn parse(alloc: Allocator, filePath: []const u8) !ParsedFile {
    const name = std.fs.path.basename(filePath);
    const stem = std.fs.path.stem(filePath);
    const upper = try std.ascii.allocUpperString(alloc, stem);
    defer alloc.free(upper);

    const result = try key.parse(alloc, upper);
    const parsedKey = switch (result.value) {
        .ok => |k| blk: {
            if (k == .normal) break :blk domain.jav.Key{
                .normal = try alloc.dupe(u8, k.normal),
            };
            break :blk k;
        },
        .err => return error.ParseKeyFailed,
    };

    return .{
        .alloc = alloc,
        .name = try alloc.dupe(u8, name),
        .key = parsedKey,
    };
}

fn toFc2(number: []const u8) domain.jav.Key {
    return .{
        .fc2 = number,
    };
}

fn toJav(pair: anytype) domain.jav.Key {
    return .{
        .jav = .{
            .id = pair[0],
            .number = pair[1],
        },
    };
}

fn toNormal(title: []const u8) domain.jav.Key {
    return .{
        .normal = title,
    };
}

const divider = mecha.oneOf(.{
    mecha.many(mecha.ascii.whitespace, .{ .min = 1, .collect = false }),
    mecha.many(mecha.ascii.char('-'), .{ .min = 1, .collect = false }),
    mecha.many(mecha.ascii.char('_'), .{ .min = 1, .collect = false }),
});
const maybeDivider = mecha.opt(divider);

const fc2 = mecha.oneOf(.{
    mecha.combine(.{
        mecha.string("FC2").discard(),
        maybeDivider.discard(),
        mecha.many(mecha.ascii.digit(10), .{ .min = 1 }),
    }).map(toFc2),
    mecha.combine(.{
        mecha.string("FC2").discard(),
        maybeDivider.discard(),
        mecha.string("PPV").discard(),
        maybeDivider.discard(),
        mecha.many(mecha.ascii.digit(10), .{ .min = 1 }),
    }).map(toFc2),
});

const jav = mecha.combine(.{
    mecha.many(mecha.ascii.alphabetic, .{ .min = 1 }),
    maybeDivider.discard(),
    mecha.many(mecha.ascii.digit(10), .{ .min = 1 }),
}).map(toJav);

const normal = mecha.rest.map(toNormal);

const key = mecha.oneOf(.{
    fc2,
    jav,
    normal,
});

test "Parse jav worked" {
    const alloc = std.testing.allocator;

    const expect = domain.jav.Key{
        .jav = .{
            .id = "STARS",
            .number = "804",
        },
    };
    const actual = (try key.parse(alloc, "STARS-804")).value.ok;
    std.debug.print("{}\n", .{actual});

    try std.testing.expect(std.meta.eql(actual, expect));
}
