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
