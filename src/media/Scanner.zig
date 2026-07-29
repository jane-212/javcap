const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const storage = @import("storage");

const Self = @This();

alloc: Allocator,
io: Io,
backend: storage.Storage,

pub fn init(alloc: Allocator, io: Io, s: storage.Storage) !Self {
    return .{
        .alloc = alloc,
        .io = io,
        .backend = s,
    };
}

pub fn scan(self: *const Self, alloc: Allocator, path: []const u8, exts: []const []const u8) ![]Entry {
    var walker = try self.backend.walk(self.alloc, path);
    defer walker.deinit();

    var entries: std.ArrayList(Entry) = .empty;
    errdefer for (entries.items) |*e| e.deinit(alloc);
    defer entries.deinit(self.alloc);

    while (try walker.next()) |*entry| {
        if (entry.fileType != .file) continue;

        const p = entry.path;

        const extWithDot = std.fs.path.extension(p);
        const ext = if (extWithDot.len == 0) extWithDot else extWithDot[1..];
        if (!matchExts(ext, exts)) continue;

        const absolutePath = try alloc.dupe(u8, p);
        errdefer alloc.free(absolutePath);

        try entries.append(self.alloc, .{
            .path = absolutePath,
        });
    }

    return entries.toOwnedSlice(alloc);
}

pub const Entry = struct {
    path: []const u8,

    pub fn deinit(self: *Entry, alloc: Allocator) void {
        alloc.free(self.path);
        self.* = undefined;
    }
};

pub fn matchExts(ext: []const u8, exts: []const []const u8) bool {
    for (exts) |e| {
        if (std.ascii.eqlIgnoreCase(ext, e)) return true;
    }
    return false;
}
