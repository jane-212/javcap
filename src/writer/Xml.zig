const std = @import("std");
const domain = @import("domain");

pub fn format(w: *std.Io.Writer, nfo: *const domain.Nfo) !void {
    try w.writeAll("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>");
    try w.writeAll("<movie>");
    if (nfo.title) |t| try writeTag(w, "title", t);
    if (nfo.originalTitle) |t| try writeTag(w, "originaltitle", t);
    if (nfo.rating) |r| try w.print("<rating>{:.1}</rating>", .{r});
    if (nfo.plot) |p| try writeTag(w, "plot", p);
    if (nfo.runtime) |r| try w.print("<runtime>{}</runtime>", .{r});
    if (nfo.mpaa) |m| switch (m) {
        .g => try w.writeAll("<mpaa>G</mpaa>"),
        .pg => try w.writeAll("<mpaa>PG</mpaa>"),
        .pg13 => try w.writeAll("<mpaa>PG-13</mpaa>"),
        .r => try w.writeAll("<mpaa>R</mpaa>"),
        .nc17 => try w.writeAll("<mpaa>NC-17</mpaa>"),
    };
    if (nfo.id) |i| try writeId(w, i);
    for (nfo.genres.items) |g| try writeTag(w, "genre", g);
    for (nfo.tags.items) |t| try writeTag(w, "tag", t);
    if (nfo.country) |c| switch (c) {
        .cn => try w.writeAll("<country>国产</country>"),
        .jp => try w.writeAll("<country>日本</country>"),
        .us => try w.writeAll("<country>欧美</country>"),
    };
    if (nfo.director) |d| try writeTag(w, "director", d);
    if (nfo.premiered) |p| try writeTag(w, "premiered", p);
    if (nfo.studio) |s| try writeTag(w, "studio", s);
    for (nfo.actresses.items) |a| {
        try w.writeAll("<actor>");
        try writeTag(w, "name", a.name);
        if (a.thumb) |t| try writeTag(w, "thumb", t);
        try w.writeAll("</actor>");
    }
    try w.writeAll("</movie>\n");
}

fn writeTag(w: *std.Io.Writer, name: []const u8, value: []const u8) !void {
    try w.writeByte('<');
    try w.writeAll(name);
    try w.writeByte('>');

    try escapeWrite(w, value);

    try w.writeAll("</");
    try w.writeAll(name);
    try w.writeByte('>');
}

fn writeId(w: *std.Io.Writer, id: []const u8) !void {
    try w.writeAll("<uniqueid type=\"num\" default=\"true\">");

    try escapeWrite(w, id);

    try w.writeAll("</uniqueid>");
}

fn escapeWrite(w: *std.Io.Writer, slice: []const u8) !void {
    for (slice) |c| {
        switch (c) {
            '&' => try w.writeAll("&amp;"),
            '<' => try w.writeAll("&lt;"),
            '>' => try w.writeAll("&gt;"),
            '"' => try w.writeAll("&quot;"),
            '\'' => try w.writeAll("&apos;"),
            else => try w.writeByte(c),
        }
    }
}

fn writeToString(nfo: *const domain.Nfo) ![]const u8 {
    var buf: std.ArrayList(u8) = .empty;
    var aw = std.Io.Writer.Allocating.fromArrayList(std.testing.allocator, &buf);
    errdefer aw.deinit();
    try format(&aw.writer, nfo);
    return aw.toOwnedSlice();
}

test "format — empty Nfo produces XML declaration and empty movie tag" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie></movie>\n",
        xml,
    );
}

test "format — writes title when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "Test Movie");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><title>Test Movie</title></movie>\n",
        xml,
    );
}

test "format — writes originalTitle when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.originalTitle = try alloc.dupe(u8, "オリジナル");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><originaltitle>オリジナル</originaltitle></movie>\n",
        xml,
    );
}

test "format — writes rating formatted to one decimal" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.rating = 8.5;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><rating>8.5</rating></movie>\n",
        xml,
    );
}

test "format — rating rounds to one decimal" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.rating = 7.99;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><rating>8.0</rating></movie>\n",
        xml,
    );
}

test "format — writes plot when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.plot = try alloc.dupe(u8, "A long story.");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><plot>A long story.</plot></movie>\n",
        xml,
    );
}

test "format — writes runtime when present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.runtime = 120;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><runtime>120</runtime></movie>\n",
        xml,
    );
}

test "format — writes mpaa G" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .g;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><mpaa>G</mpaa></movie>\n",
        xml,
    );
}

test "format — writes mpaa PG" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .pg;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><mpaa>PG</mpaa></movie>\n",
        xml,
    );
}

test "format — writes mpaa PG-13" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .pg13;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><mpaa>PG-13</mpaa></movie>\n",
        xml,
    );
}

test "format — writes mpaa R" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .r;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><mpaa>R</mpaa></movie>\n",
        xml,
    );
}

test "format — writes mpaa NC-17" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.mpaa = .nc17;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><mpaa>NC-17</mpaa></movie>\n",
        xml,
    );
}

test "format — writes uniqueid when id present" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.id = try alloc.dupe(u8, "STARS-804");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><uniqueid type=\"num\" default=\"true\">STARS-804</uniqueid></movie>\n",
        xml,
    );
}

test "format — writes genres" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.genres.append(alloc, try alloc.dupe(u8, "Action"));
    try nfo.genres.append(alloc, try alloc.dupe(u8, "Drama"));

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><genre>Action</genre><genre>Drama</genre></movie>\n",
        xml,
    );
}

test "format — writes tags" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.tags.append(alloc, try alloc.dupe(u8, "HD"));
    try nfo.tags.append(alloc, try alloc.dupe(u8, "字幕"));

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><tag>HD</tag><tag>字幕</tag></movie>\n",
        xml,
    );
}

test "format — writes country CN" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.country = .cn;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><country>国产</country></movie>\n",
        xml,
    );
}

test "format — writes country JP" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.country = .jp;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><country>日本</country></movie>\n",
        xml,
    );
}

test "format — writes country US" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.country = .us;

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><country>欧美</country></movie>\n",
        xml,
    );
}

test "format — writes director" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.director = try alloc.dupe(u8, "Spielberg");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><director>Spielberg</director></movie>\n",
        xml,
    );
}

test "format — writes premiered" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.premiered = try alloc.dupe(u8, "2024-01-15");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><premiered>2024-01-15</premiered></movie>\n",
        xml,
    );
}

test "format — writes studio" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.studio = try alloc.dupe(u8, "Paramount");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><studio>Paramount</studio></movie>\n",
        xml,
    );
}

test "format — writes actress without thumb" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Jane Doe") });

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><actor><name>Jane Doe</name></actor></movie>\n",
        xml,
    );
}

test "format — writes actress with thumb" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{
        .name = try alloc.dupe(u8, "Jane Doe"),
        .thumb = try alloc.dupe(u8, "/thumbs/jane.jpg"),
    });

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><actor><name>Jane Doe</name><thumb>/thumbs/jane.jpg</thumb></actor></movie>\n",
        xml,
    );
}

test "format — writes multiple actresses" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Alice") });
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Bob"), .thumb = try alloc.dupe(u8, "/t/bob.jpg") });

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><actor><name>Alice</name></actor><actor><name>Bob</name><thumb>/t/bob.jpg</thumb></actor></movie>\n",
        xml,
    );
}

test "format — escapes & < > \" ' in text content" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "A & B < C > D \"E\" 'F'");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><title>A &amp; B &lt; C &gt; D &quot;E&quot; &apos;F&apos;</title></movie>\n",
        xml,
    );
}

test "format — escapes special characters in id" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.id = try alloc.dupe(u8, "id<1>&2");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><uniqueid type=\"num\" default=\"true\">id&lt;1&gt;&amp;2</uniqueid></movie>\n",
        xml,
    );
}

test "format — escapes special characters in genre" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.genres.append(alloc, try alloc.dupe(u8, "Sci-Fi & Fantasy"));

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><genre>Sci-Fi &amp; Fantasy</genre></movie>\n",
        xml,
    );
}

test "format — escapes special characters in actress name and thumb" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    try nfo.actresses.append(alloc, .{
        .name = try alloc.dupe(u8, "O'Brien"),
        .thumb = try alloc.dupe(u8, "/thumb/a&b.jpg"),
    });

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><actor><name>O&apos;Brien</name><thumb>/thumb/a&amp;b.jpg</thumb></actor></movie>\n",
        xml,
    );
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
    try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, "Alice"), .thumb = try alloc.dupe(u8, "/a.jpg") });

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>" ++
            "<movie>" ++
            "<title>Test</title>" ++
            "<originaltitle>テスト</originaltitle>" ++
            "<rating>9.2</rating>" ++
            "<plot>A story.</plot>" ++
            "<runtime>90</runtime>" ++
            "<mpaa>R</mpaa>" ++
            "<uniqueid type=\"num\" default=\"true\">ID-001</uniqueid>" ++
            "<genre>Action</genre>" ++
            "<tag>HD</tag>" ++
            "<country>日本</country>" ++
            "<director>Dir</director>" ++
            "<premiered>2024-06-01</premiered>" ++
            "<studio>Studio X</studio>" ++
            "<actor><name>Alice</name><thumb>/a.jpg</thumb></actor>" ++
            "</movie>\n",
        xml,
    );
}

test "format — omits null optional fields" {
    const alloc = std.testing.allocator;
    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();
    nfo.title = try alloc.dupe(u8, "Just Title");

    const xml = try writeToString(&nfo);
    defer alloc.free(xml);

    try std.testing.expectEqualStrings(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?><movie><title>Just Title</title></movie>\n",
        xml,
    );
}
