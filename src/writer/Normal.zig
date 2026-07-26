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
    for (nfo.actresses.items) |a| {
        try w.writeAll("actor:");
        try w.print(" {s}", .{a.name});
        if (a.thumb) |t| try w.print(" {s}", .{t});
        try w.writeAll("\n");
    }
}
