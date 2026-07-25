const std = @import("std");
const Allocator = std.mem.Allocator;
const domain = @import("domain");

const Self = @This();

pub const ParsedFile = struct {
    alloc: Allocator,
    key: domain.jav.Key,
    extension: []const u8,

    pub fn deinit(self: *ParsedFile) void {
        self.alloc.free(self.extension);
        self.key.deinit(self.alloc);
        self.* = undefined;
    }
};

pub fn parse(alloc: Allocator, filePath: []const u8) !ParsedFile {
    const extension = std.fs.path.extension(filePath);
    const stem = std.fs.path.stem(filePath);
    const upper = try std.ascii.allocUpperString(alloc, stem);
    defer alloc.free(upper);

    return .{
        .alloc = alloc,
        .extension = alloc.dupe(u8, extension),
    };
}
