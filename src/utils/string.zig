const std = @import("std");
const Allocator = std.mem.Allocator;

pub fn trimAll(alloc: Allocator, slice: []const u8, values_to_strip: []const u8) ![]const u8 {
    const trimmed = std.mem.trim(u8, slice, values_to_strip);

    var buffer: std.ArrayList(u8) = .empty;
    defer buffer.deinit(alloc);

    var it = std.mem.tokenizeAny(u8, trimmed, values_to_strip);
    while (it.next()) |part| {
        if (buffer.items.len > 0) try buffer.append(alloc, ' ');
        try buffer.appendSlice(alloc, part);
    }

    return try buffer.toOwnedSlice(alloc);
}

test "trimAll — basic space trimming and collapsing" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "  hello  world  ", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello world", result);
}

test "trimAll — empty string" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("", result);
}

test "trimAll — all trim characters" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "   ", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("", result);
}

test "trimAll — no trimming needed" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "hello world", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello world", result);
}

test "trimAll — multi-char trim set with tabs" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, " \t hello \t world \t", " \t");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello world", result);
}

test "trimAll — newlines as trim characters" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "\n\nhello\nworld\n\n", "\n");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello world", result);
}

test "trimAll — single word with surrounding spaces" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "  hello  ", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello", result);
}

test "trimAll — collapse multiple different whitespace chars in middle" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "hello \t  \t world", " \t");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello world", result);
}

test "trimAll — comma delimiter compaction" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, ",,a,,b,,c,,", ",");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("a b c", result);
}

test "trimAll — adjacent words separated by single trim char" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "a b c", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("a b c", result);
}

test "trimAll — trim set not present in string" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "hello world", ",");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello world", result);
}

test "trimAll — mixed characters, trim only whitespace" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "  hello, world!  ", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("hello, world!", result);
}

test "trimAll — single character string" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, "a", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("a", result);
}

test "trimAll — single trim character" {
    const alloc = std.testing.allocator;
    const result = try trimAll(alloc, " ", " ");
    defer alloc.free(result);
    try std.testing.expectEqualStrings("", result);
}
