const std = @import("std");
const Allocator = std.mem.Allocator;
const domain = @import("domain");

/// 配置中支持的元数据占位符（以 "." 开头）。
pub const token_names = [_][]const u8{
    ".title",
    ".originalTitle",
    ".id",
    ".studio",
    ".director",
    ".premiered",
    ".plot",
    ".country",
    ".rating",
    ".actress",
};

pub fn isKnownToken(segment: []const u8) bool {
    for (token_names) |token| {
        if (std.mem.eql(u8, token, segment)) return true;
    }
    return false;
}

/// 把 root 与 segments 拼接成最终输出目录。
/// 以 "." 开头的段按元数据占位符替换，缺失或替换结果为空时省略该段；
/// 其余段按字面量使用。所有段都被替换或省略时回退为 root 本身。
pub fn resolve(
    alloc: Allocator,
    root: []const u8,
    segments: []const []const u8,
    nfo: *const domain.Nfo,
) ![]const u8 {
    var parts: std.ArrayList([]const u8) = .empty;
    defer parts.deinit(alloc);
    try parts.append(alloc, root);

    const OwnedPair = struct {
        value: []const u8,
        cleaned: []const u8,
    };

    var owned: std.ArrayList(OwnedPair) = .empty;
    defer {
        for (owned.items) |pair| {
            alloc.free(pair.value);
            alloc.free(pair.cleaned);
        }
        owned.deinit(alloc);
    }

    for (segments) |segment| {
        if (segment.len == 0) continue;

        if (segment[0] != '.') {
            // 字面量段：配置校验已保证不含路径分隔符且不是 . / ..
            if (std.mem.eql(u8, segment, ".") or std.mem.eql(u8, segment, "..")) continue;
            try parts.append(alloc, segment);
            continue;
        }

        const value = try resolveToken(alloc, segment, nfo) orelse try alloc.dupe(u8, "unknown");
        errdefer alloc.free(value);

        const cleaned = try sanitize(alloc, value);
        errdefer alloc.free(cleaned);

        if (cleaned.len == 0 or
            std.mem.eql(u8, cleaned, ".") or
            std.mem.eql(u8, cleaned, ".."))
        {
            alloc.free(value);
            alloc.free(cleaned);
            continue;
        }

        try parts.append(alloc, cleaned);
        try owned.append(alloc, .{ .value = value, .cleaned = cleaned });
    }

    return std.fs.path.join(alloc, parts.items);
}

/// 解析单个占位符；元数据缺失时返回 null。
fn resolveToken(
    alloc: Allocator,
    token: []const u8,
    nfo: *const domain.Nfo,
) !?[]const u8 {
    if (std.mem.eql(u8, token, ".title")) return if (nfo.title) |v| try alloc.dupe(u8, v) else null;
    if (std.mem.eql(u8, token, ".originalTitle")) return if (nfo.originalTitle) |v| try alloc.dupe(u8, v) else null;
    if (std.mem.eql(u8, token, ".id")) return if (nfo.id) |v| try alloc.dupe(u8, v) else null;
    if (std.mem.eql(u8, token, ".studio")) return if (nfo.studio) |v| try alloc.dupe(u8, v) else null;
    if (std.mem.eql(u8, token, ".director")) return if (nfo.director) |v| try alloc.dupe(u8, v) else null;
    if (std.mem.eql(u8, token, ".premiered")) return if (nfo.premiered) |v| try alloc.dupe(u8, v) else null;
    if (std.mem.eql(u8, token, ".plot")) return if (nfo.plot) |v| try alloc.dupe(u8, v) else null;
    if (std.mem.eql(u8, token, ".country")) {
        const country = nfo.country orelse return null;
        return try alloc.dupe(u8, switch (country) {
            .cn => "cn",
            .jp => "jp",
            .us => "us",
        });
    }
    if (std.mem.eql(u8, token, ".rating")) {
        const rating = nfo.rating orelse return null;
        return try std.fmt.allocPrint(alloc, "{:.1}", .{rating});
    }
    if (std.mem.eql(u8, token, ".actress")) {
        if (nfo.actresses.items.len == 0) return null;
        return try alloc.dupe(u8, nfo.actresses.items[0].name);
    }
    return error.UnknownToken;
}

/// 把目录段中非法文件名/路径字符替换为 "-"。
fn sanitize(alloc: Allocator, value: []const u8) ![]const u8 {
    var needs_sanitize = false;
    for (value) |c| {
        if (isIllegalPathChar(c)) {
            needs_sanitize = true;
            break;
        }
    }
    if (!needs_sanitize) return alloc.dupe(u8, value);

    var result: std.ArrayList(u8) = .empty;
    errdefer result.deinit(alloc);
    for (value) |c| {
        try result.append(alloc, if (isIllegalPathChar(c)) '-' else c);
    }
    return result.toOwnedSlice(alloc);
}

fn isIllegalPathChar(c: u8) bool {
    return switch (c) {
        '\\', '/', ':', '*', '?', '"', '<', '>', '|' => true,
        else => c < 0x20 or c == 0x7f,
    };
}

const testing = std.testing;

fn testNfo(alloc: Allocator) !domain.Nfo {
    return domain.Nfo.init(alloc);
}

test "resolve — literal segments only" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();

    const result = try resolve(alloc, "/out", &.{ "hello", "1" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/hello/1", result);
}

test "resolve — replaces title and id, joins with to" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "初恋");
    nfo.id = try alloc.dupe(u8, "STARS-804");

    const result = try resolve(alloc, "/out", &.{ "hello", ".title", ".id", "1" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/hello/初恋/STARS-804/1", result);
}

test "resolve — missing field is omitted" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();
    nfo.id = try alloc.dupe(u8, "ABC");

    const result = try resolve(alloc, "/out", &.{ "a", ".title", ".id" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/a/ABC", result);
}

test "resolve — all tokens missing falls back to root" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();

    const result = try resolve(alloc, "/out", &.{ ".title", ".studio" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out", result);
}

test "resolve — empty segments list returns root" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();

    const result = try resolve(alloc, "/out", &.{}, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out", result);
}

test "resolve — sanitizes illegal characters in metadata" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "a/b:c*d");
    nfo.id = try alloc.dupe(u8, "ABC");

    const result = try resolve(alloc, "/out", &.{ ".title", ".id" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/a-b-c-d/ABC", result);
}

test "resolve — country uses lowercase tag" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();
    nfo.country = .jp;
    nfo.id = try alloc.dupe(u8, "ABC");

    const result = try resolve(alloc, "/out", &.{ ".country", ".id" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/jp/ABC", result);
}

test "resolve — rating formatted to one decimal" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();
    nfo.rating = 7.99;
    nfo.id = try alloc.dupe(u8, "ABC");

    const result = try resolve(alloc, "/out", &.{ ".rating", ".id" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/8.0/ABC", result);
}

test "resolve — actress uses first name" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Alice") });
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Bob") });
    nfo.id = try alloc.dupe(u8, "ABC");

    const result = try resolve(alloc, "/out", &.{ ".actress", ".id" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/Alice/ABC", result);
}

test "resolve — unknown tokens are an error" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();

    for ([_][]const u8{ ".foo", ".key" }) |bad| {
        try testing.expectError(error.UnknownToken, resolve(alloc, "/out", &.{bad}, &nfo));
    }
}

test "resolve — token value that becomes dot is omitted" {
    const alloc = testing.allocator;
    var nfo = try testNfo(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, ".");
    nfo.id = try alloc.dupe(u8, "ABC");

    const result = try resolve(alloc, "/out", &.{ ".title", ".id" }, &nfo);
    defer alloc.free(result);

    try testing.expectEqualStrings("/out/ABC", result);
}
