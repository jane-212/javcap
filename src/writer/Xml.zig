const std = @import("std");
const domain = @import("domain");

pub fn format(w: *std.Io.Writer, nfo: *const domain.Nfo) !void {
    try w.writeAll("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>");
    try w.writeAll("<movie>");
    if (nfo.title) |t| try w.print("<title>{s}</title>", .{t});
    if (nfo.originalTitle) |t| try w.print("<originaltitle>{s}</originaltitle>", .{t});
    if (nfo.rating) |r| try w.print("<rating>{:.1}</rating>", .{r});
    if (nfo.plot) |p| try w.print("<plot>{s}</plot>", .{p});
    if (nfo.runtime) |r| try w.print("<runtime>{}</runtime>", .{r});
    if (nfo.mpaa) |m| switch (m) {
        .g => try w.writeAll("<mpaa>G</mpaa>"),
        .pg => try w.writeAll("<mpaa>PG</mpaa>"),
        .pg13 => try w.writeAll("<mpaa>PG-13</mpaa>"),
        .r => try w.writeAll("<mpaa>R</mpaa>"),
        .nc17 => try w.writeAll("<mpaa>NC-17</mpaa>"),
    };
    if (nfo.id) |i| try w.print("<uniqueid type=\"num\" default=\"true\">{s}</uniqueid>", .{i});
    for (nfo.genres.items) |g| try w.print("<genre>{s}</genre>", .{g});
    for (nfo.tags.items) |t| try w.print("<tag>{s}</tag>", .{t});
    if (nfo.country) |c| switch (c) {
        .cn => try w.writeAll("<country>国产</country>"),
        .jp => try w.writeAll("<country>日本</country>"),
        .us => try w.writeAll("<country>欧美</country>"),
    };
    if (nfo.director) |d| try w.print("<director>{s}</director>", .{d});
    if (nfo.premiered) |p| try w.print("<premiered>{s}</premiered>", .{p});
    if (nfo.studio) |s| try w.print("<studio>{s}</studio>", .{s});
    for (nfo.actresses.items) |a| {
        try w.writeAll("<actor>");
        try w.print("<name>{s}</name>", .{a.name});
        if (a.thumb) |t| try w.print("<thumb>{s}</thumb>", .{t});
        try w.writeAll("</actor>");
    }
    try w.writeAll("</movie>\n");
}
