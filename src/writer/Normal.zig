const std = @import("std");
const domain = @import("domain");

pub fn format(w: *std.Io.Writer, nfo: *const domain.Nfo) !void {
    if (nfo.title) |t| try w.print("title: {s}\n", .{t});
    if (nfo.originalTitle) |t| try w.print("originaltitle: {s}\n", .{t});
    if (nfo.rating) |r| try w.print("rating: {:.1}\n", .{r});
    if (nfo.plot) |p| try w.print("plot: {s}\n", .{p});
    if (nfo.runtime) |r| try w.print("runtime: {}\n", .{r});
    if (nfo.mpaa) |m| switch (m) {
        .g => try w.writeAll("mpaa: G\n"),
        .pg => try w.writeAll("mpaa: PG\n"),
        .pg13 => try w.writeAll("mpaa: PG-13\n"),
        .r => try w.writeAll("mpaa: R\n"),
        .nc17 => try w.writeAll("mpaa: NC-17\n"),
    };
    if (nfo.id) |i| try w.print("id: {s}\n", .{i});
    if (nfo.genres.items.len > 0) {
        try w.writeAll("genres:");
        for (nfo.genres.items) |g| try w.print(" {s}", .{g});
        try w.writeAll("\n");
    }
    if (nfo.tags.items.len > 0) {
        try w.writeAll("tags:");
        for (nfo.tags.items) |t| try w.print(" {s}", .{t});
        try w.writeAll("\n");
    }
    if (nfo.country) |c| switch (c) {
        .cn => try w.writeAll("country: 国产\n"),
        .jp => try w.writeAll("country: 日本\n"),
        .us => try w.writeAll("country: 欧美\n"),
    };
    if (nfo.director) |d| try w.print("director: {s}\n", .{d});
    if (nfo.premiered) |p| try w.print("premiered: {s}\n", .{p});
    if (nfo.studio) |s| try w.print("studio: {s}\n", .{s});
    if (nfo.poster) |p| try w.print("poster: {} bytes\n", .{p.len});
    if (nfo.fanart) |f| try w.print("fanart: {} bytes\n", .{f.len});
    if (nfo.subtitle) |s| try w.print("subtitle: {} bytes\n", .{s.len});
    for (nfo.actresses.items) |a| {
        try w.writeAll("actor:");
        try w.print(" {s}", .{a.name});
        if (a.thumb) |t| try w.print(" {s}", .{t});
        try w.writeAll("\n");
    }
}

fn writeToString(nfo: *const domain.Nfo) ![]const u8 {
    var buf: std.ArrayList(u8) = .empty;
    var aw = std.Io.Writer.Allocating.fromArrayList(std.testing.allocator, &buf);
    errdefer aw.deinit();
    try format(&aw.writer, nfo);
    return aw.toOwnedSlice();
}

test "format — empty Nfo produces empty string" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("", result);
}

test "format — writes title when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "Test Movie");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("title: Test Movie\n", result);
}

test "format — writes originalTitle when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.originalTitle = try alloc.dupe(u8, "オリジナル");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("originaltitle: オリジナル\n", result);
}

test "format — writes rating formatted to one decimal" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.rating = 8.5;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("rating: 8.5\n", result);
}

test "format — rating rounds to one decimal" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.rating = 7.99;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("rating: 8.0\n", result);
}

test "format — writes plot when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.plot = try alloc.dupe(u8, "A long story.");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("plot: A long story.\n", result);
}

test "format — writes runtime when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.runtime = 120;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("runtime: 120\n", result);
}

test "format — writes mpaa G" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .g;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("mpaa: G\n", result);
}

test "format — writes mpaa PG" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .pg;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("mpaa: PG\n", result);
}

test "format — writes mpaa PG-13" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .pg13;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("mpaa: PG-13\n", result);
}

test "format — writes mpaa R" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .r;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("mpaa: R\n", result);
}

test "format — writes mpaa NC-17" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .nc17;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("mpaa: NC-17\n", result);
}

test "format — writes id when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.id = try alloc.dupe(u8, "STARS-804");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("id: STARS-804\n", result);
}

test "format — writes genres" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.genres.append(alloc, try alloc.dupe(u8, "Action"));
    try nfo.genres.append(alloc, try alloc.dupe(u8, "Drama"));

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("genres: Action Drama\n", result);
}

test "format — writes tags" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.tags.append(alloc, try alloc.dupe(u8, "HD"));
    try nfo.tags.append(alloc, try alloc.dupe(u8, "字幕"));

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("tags: HD 字幕\n", result);
}

test "format — writes country CN" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.country = .cn;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("country: 国产\n", result);
}

test "format — writes country JP" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.country = .jp;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("country: 日本\n", result);
}

test "format — writes country US" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.country = .us;

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("country: 欧美\n", result);
}

test "format — writes director" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.director = try alloc.dupe(u8, "Spielberg");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("director: Spielberg\n", result);
}

test "format — writes premiered" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.premiered = try alloc.dupe(u8, "2024-01-15");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("premiered: 2024-01-15\n", result);
}

test "format — writes studio" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.studio = try alloc.dupe(u8, "Paramount");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("studio: Paramount\n", result);
}

test "format — writes poster with byte length" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.poster = try alloc.dupe(u8, "https://example.com/poster.jpg");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("poster: 30 bytes\n", result);
}

test "format — writes fanart with byte length" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.fanart = try alloc.dupe(u8, "https://example.com/fanart.jpg");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("fanart: 30 bytes\n", result);
}

test "format — writes subtitle with byte length" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.subtitle = try alloc.dupe(u8, "https://example.com/subtitle.srt");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("subtitle: 32 bytes\n", result);
}

test "format — writes poster byte length for binary data" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.poster = try alloc.dupe(u8, &[_]u8{ 0x00, 0x01, 0x02, 0xFF });

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("poster: 4 bytes\n", result);
}

test "format — writes actress without thumb" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Jane Doe") });

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("actor: Jane Doe\n", result);
}

test "format — writes actress with thumb" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{
        .name = try alloc.dupe(u8, "Jane Doe"),
        .thumb = try alloc.dupe(u8, "/thumbs/jane.jpg"),
    });

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("actor: Jane Doe /thumbs/jane.jpg\n", result);
}

test "format — writes multiple actresses" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Alice") });
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Bob"), .thumb = try alloc.dupe(u8, "/t/bob.jpg") });

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("actor: Alice\nactor: Bob /t/bob.jpg\n", result);
}

test "format — full Nfo with all fields" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "Test");
    nfo.originalTitle = try alloc.dupe(u8, "テスト");
    nfo.rating = 9.2;
    nfo.plot = try alloc.dupe(u8, "A story.");
    nfo.runtime = 90;
    nfo.mpaa = .r;
    nfo.id = try alloc.dupe(u8, "ID-001");
    try nfo.genres.append(alloc, try alloc.dupe(u8, "Action"));
    try nfo.tags.append(alloc, try alloc.dupe(u8, "HD"));
    nfo.country = .jp;
    nfo.director = try alloc.dupe(u8, "Dir");
    nfo.premiered = try alloc.dupe(u8, "2024-06-01");
    nfo.studio = try alloc.dupe(u8, "Studio X");
    nfo.poster = try alloc.dupe(u8, "https://example.com/poster.jpg");
    nfo.fanart = try alloc.dupe(u8, "https://example.com/fanart.jpg");
    nfo.subtitle = try alloc.dupe(u8, "https://example.com/subtitle.srt");
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Alice"), .thumb = try alloc.dupe(u8, "/a.jpg") });

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings(
        "title: Test\n" ++
            "originaltitle: テスト\n" ++
            "rating: 9.2\n" ++
            "plot: A story.\n" ++
            "runtime: 90\n" ++
            "mpaa: R\n" ++
            "id: ID-001\n" ++
            "genres: Action\n" ++
            "tags: HD\n" ++
            "country: 日本\n" ++
            "director: Dir\n" ++
            "premiered: 2024-06-01\n" ++
            "studio: Studio X\n" ++
            "poster: 30 bytes\n" ++
            "fanart: 30 bytes\n" ++
            "subtitle: 32 bytes\n" ++
            "actor: Alice /a.jpg\n",
        result,
    );
}

test "format — omits null optional fields" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "Just Title");

    const result = try writeToString(&nfo);
    defer alloc.free(result);

    try std.testing.expectEqualStrings("title: Just Title\n", result);
}
