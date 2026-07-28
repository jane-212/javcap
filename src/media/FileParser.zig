const std = @import("std");
const Allocator = std.mem.Allocator;
const domain = @import("domain");
const mecha = @import("mecha");

const Self = @This();

pub const ParsedFile = struct {
    alloc: Allocator,
    key: domain.jav.Key,
    ext: []const u8,

    pub fn deinit(self: *ParsedFile) void {
        self.alloc.free(self.ext);
        self.key.deinit(self.alloc);
        self.* = undefined;
    }
};

pub fn parse(alloc: Allocator, filePath: []const u8) !ParsedFile {
    const ext = std.fs.path.extension(filePath);
    const stem = std.fs.path.stem(filePath);
    const upper = try std.ascii.allocUpperString(alloc, stem);
    defer alloc.free(upper);

    const result = try key.parse(alloc, upper);
    const parsedKey = switch (result.value) {
        .ok => |k| switch (k) {
            .normal => |n| domain.jav.Key{ .normal = try alloc.dupe(u8, n) },
            .fc2 => |f| domain.jav.Key{ .fc2 = try alloc.dupe(u8, f) },
            .jav => |j| domain.jav.Key{ .jav = .{
                .id = try alloc.dupe(u8, j.id),
                .number = try alloc.dupe(u8, j.number),
            } },
        },
        .err => return error.ParseKeyFailed,
    };

    return .{
        .alloc = alloc,
        .ext = try alloc.dupe(u8, ext),
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
        mecha.many(mecha.ascii.digit(10), .{ .min = 1, .collect = false }),
    }).map(toFc2),
    mecha.combine(.{
        mecha.string("FC2").discard(),
        maybeDivider.discard(),
        mecha.string("PPV").discard(),
        maybeDivider.discard(),
        mecha.many(mecha.ascii.digit(10), .{ .min = 1, .collect = false }),
    }).map(toFc2),
});

const jav = mecha.combine(.{
    mecha.many(mecha.ascii.alphabetic, .{ .min = 1, .collect = false }),
    maybeDivider.discard(),
    mecha.many(mecha.ascii.digit(10), .{ .min = 1, .collect = false }),
}).map(toJav);

const normal = mecha.rest.map(toNormal);

const key = mecha.oneOf(.{
    fc2,
    jav,
    normal,
});

test "Parse jav with dash separator" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "STARS-804");
    const actual = result.value.ok;

    try std.testing.expect(actual == .jav);
    try std.testing.expectEqualStrings("STARS", actual.jav.id);
    try std.testing.expectEqualStrings("804", actual.jav.number);
}

test "Parse jav with underscore separator" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "ABC_123");
    const actual = result.value.ok;

    try std.testing.expect(actual == .jav);
    try std.testing.expectEqualStrings("ABC", actual.jav.id);
    try std.testing.expectEqualStrings("123", actual.jav.number);
}

test "Parse jav with whitespace separator" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "ABC 123");
    const actual = result.value.ok;

    try std.testing.expect(actual == .jav);
    try std.testing.expectEqualStrings("ABC", actual.jav.id);
    try std.testing.expectEqualStrings("123", actual.jav.number);
}

test "Parse jav with multiple separators" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "X--99");
    const actual = result.value.ok;

    try std.testing.expect(actual == .jav);
    try std.testing.expectEqualStrings("X", actual.jav.id);
    try std.testing.expectEqualStrings("99", actual.jav.number);
}

test "Parse jav single character id" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "A-1");
    const actual = result.value.ok;

    try std.testing.expect(actual == .jav);
    try std.testing.expectEqualStrings("A", actual.jav.id);
    try std.testing.expectEqualStrings("1", actual.jav.number);
}

test "Parse jav without separator" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "ABCDEF123");
    const actual = result.value.ok;

    try std.testing.expect(actual == .jav);
    try std.testing.expectEqualStrings("ABCDEF", actual.jav.id);
    try std.testing.expectEqualStrings("123", actual.jav.number);
}

test "Parse fc2 with dash" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2-12345");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("12345", actual.fc2);
}

test "Parse fc2 with ppv" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2-PPV-12345");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("12345", actual.fc2);
}

test "Parse fc2 ppv without separator" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2PPV12345");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("12345", actual.fc2);
}

test "Parse fc2 ppv with mixed separators" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2-PPV_67890");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("67890", actual.fc2);
}

test "Parse fc2 with underscore" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2_99999");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("99999", actual.fc2);
}

test "Parse fc2 takes priority over jav" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2-123");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("123", actual.fc2);
}

test "Parse normal fallback for non-parseable name" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "JustATitle");
    const actual = result.value.ok;

    try std.testing.expect(actual == .normal);
    try std.testing.expectEqualStrings("JustATitle", actual.normal);
}

test "Parse normal for digits-only after alphabetic fails" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "123-456");
    const actual = result.value.ok;

    try std.testing.expect(actual == .normal);
    try std.testing.expectEqualStrings("123-456", actual.normal);
}

test "Parse fc2 with spaces" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2 55555");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("55555", actual.fc2);
}

test "Parse fc2 ppv with spaces" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2 PPV 55555");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("55555", actual.fc2);
}

test "Parse jav with leading whitespace not consumed" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, " ABC-123");
    const actual = result.value.ok;

    try std.testing.expect(actual == .normal);
    try std.testing.expectEqualStrings(" ABC-123", actual.normal);
}

test "Parse jav with trailing garbage is still matched" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "ABC-123X");
    const actual = result.value.ok;

    try std.testing.expect(actual == .jav);
    try std.testing.expectEqualStrings("ABC", actual.jav.id);
    try std.testing.expectEqualStrings("123", actual.jav.number);
}

test "Parse fc2 with trailing chars" {
    const alloc = std.testing.allocator;

    const result = try key.parse(alloc, "FC2-123.avi");
    const actual = result.value.ok;

    try std.testing.expect(actual == .fc2);
    try std.testing.expectEqualStrings("123", actual.fc2);
}

test "parse with jav file path" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/videos/STARS-804.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .jav);
    try std.testing.expectEqualStrings("STARS", parsed.key.jav.id);
    try std.testing.expectEqualStrings("804", parsed.key.jav.number);
    try std.testing.expectEqualStrings("STARS-804.mp4", parsed.name);
}

test "parse with fc2 file path" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/downloads/FC2-12345678.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("12345678", parsed.key.fc2);
    try std.testing.expectEqualStrings("FC2-12345678.mp4", parsed.name);
}

test "parse with fc2 ppv file path" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "FC2-PPV-99999.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("99999", parsed.key.fc2);
    try std.testing.expectEqualStrings("FC2-PPV-99999.mp4", parsed.name);
}

test "parse with normal fallback file path" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/movies/SomeFilmTitle.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .normal);
    try std.testing.expectEqualStrings("SOMEFILMTITLE", parsed.key.normal);
    try std.testing.expectEqualStrings("SomeFilmTitle.mp4", parsed.name);
}

test "parse with non-ASCII filename falls to normal" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/path/日本語.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .normal);
    try std.testing.expectEqualStrings("日本語", parsed.key.normal);
    try std.testing.expectEqualStrings("日本語.mp4", parsed.name);
}

test "parse file with multiple dots" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "movie.STARS-804.final.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .normal);
    try std.testing.expectEqualStrings("MOVIE.STARS-804.FINAL", parsed.key.normal);
    try std.testing.expectEqualStrings("movie.STARS-804.final.mp4", parsed.name);
}

test "parse preserves case in name" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/videos/Stars-804.MP4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .jav);
    try std.testing.expectEqualStrings("STARS", parsed.key.jav.id);
    try std.testing.expectEqualStrings("804", parsed.key.jav.number);
    try std.testing.expectEqualStrings("Stars-804.MP4", parsed.name);
}

test "parse fc2 with ppv and underscore in path" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/videos/FC2_PPV_77777.mkv");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("77777", parsed.key.fc2);
    try std.testing.expectEqualStrings("FC2_PPV_77777.mkv", parsed.name);
}

test "parse with directory containing dots" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/path.with.dots/STARS-804.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .jav);
    try std.testing.expectEqualStrings("STARS", parsed.key.jav.id);
    try std.testing.expectEqualStrings("804", parsed.key.jav.number);
    try std.testing.expectEqualStrings("STARS-804.mp4", parsed.name);
}

test "parse with uppercase extension" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "/videos/FC2-42.AVI");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("42", parsed.key.fc2);
    try std.testing.expectEqualStrings("FC2-42.AVI", parsed.name);
}

test "parse no-extension file" {
    const alloc = std.testing.allocator;

    var parsed = try Self.parse(alloc, "FC2-12345");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("12345", parsed.key.fc2);
    try std.testing.expectEqualStrings("FC2-12345", parsed.name);
}
