const std = @import("std");
const Allocator = std.mem.Allocator;
const domain = @import("domain");
const mecha = @import("mecha");
const media = @import("root.zig");

pub const ParsedFile = struct {
    alloc: Allocator,
    key: domain.jav.Key,
    ext: []const u8,

    pub fn deinit(self: *ParsedFile) void {
        self.alloc.free(self.ext);
        self.key.deinit(self.alloc);
    }
};

pub fn parse(alloc: Allocator, filePath: []const u8) !ParsedFile {
    const ext = std.fs.path.extension(filePath);
    const stem = std.fs.path.stem(filePath);

    var parsedKey = try media.KeyParser.parse(alloc, stem);
    errdefer parsedKey.deinit(alloc);

    return .{
        .alloc = alloc,
        .ext = try alloc.dupe(u8, ext),
        .key = parsedKey,
    };
}

test "parse with jav file path" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/videos/STARS-804.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .jav);
    try std.testing.expectEqualStrings("STARS", parsed.key.jav.id);
    try std.testing.expectEqualStrings("804", parsed.key.jav.number);
    try std.testing.expectEqualStrings(".mp4", parsed.ext);
}

test "parse with fc2 file path" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/downloads/FC2-12345678.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("12345678", parsed.key.fc2);
    try std.testing.expectEqualStrings(".mp4", parsed.ext);
}

test "parse with fc2 ppv file path" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "FC2-PPV-99999.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("99999", parsed.key.fc2);
    try std.testing.expectEqualStrings(".mp4", parsed.ext);
}

test "parse with normal fallback file path" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/movies/SomeFilmTitle.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .normal);
    try std.testing.expectEqualStrings("SOMEFILMTITLE", parsed.key.normal);
    try std.testing.expectEqualStrings(".mp4", parsed.ext);
}

test "parse with non-ASCII filename falls to normal" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/path/日本語.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .normal);
    try std.testing.expectEqualStrings("日本語", parsed.key.normal);
    try std.testing.expectEqualStrings(".mp4", parsed.ext);
}

test "parse file with multiple dots" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "movie.STARS-804.final.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .normal);
    try std.testing.expectEqualStrings("MOVIE.STARS-804.FINAL", parsed.key.normal);
    try std.testing.expectEqualStrings(".mp4", parsed.ext);
}

test "parse preserves case in name" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/videos/Stars-804.MP4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .jav);
    try std.testing.expectEqualStrings("STARS", parsed.key.jav.id);
    try std.testing.expectEqualStrings("804", parsed.key.jav.number);
    try std.testing.expectEqualStrings(".MP4", parsed.ext);
}

test "parse fc2 with ppv and underscore in path" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/videos/FC2_PPV_77777.mkv");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("77777", parsed.key.fc2);
    try std.testing.expectEqualStrings(".mkv", parsed.ext);
}

test "parse with directory containing dots" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/path.with.dots/STARS-804.mp4");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .jav);
    try std.testing.expectEqualStrings("STARS", parsed.key.jav.id);
    try std.testing.expectEqualStrings("804", parsed.key.jav.number);
    try std.testing.expectEqualStrings(".mp4", parsed.ext);
}

test "parse with uppercase extension" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "/videos/FC2-42.AVI");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("42", parsed.key.fc2);
    try std.testing.expectEqualStrings(".AVI", parsed.ext);
}

test "parse no-extension file" {
    const alloc = std.testing.allocator;

    var parsed = try parse(alloc, "FC2-12345");
    defer parsed.deinit();

    try std.testing.expect(parsed.key == .fc2);
    try std.testing.expectEqualStrings("12345", parsed.key.fc2);
    try std.testing.expectEqualStrings("", parsed.ext);
}
