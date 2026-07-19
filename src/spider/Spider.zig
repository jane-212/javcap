const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const core = @import("core");
const spider = @import("root.zig");
const domain = @import("domain");

const Self = @This();

alloc: Allocator,
io: Io,
http_client: core.HttpClient,
spiders: std.ArrayList(spider.Crawler),

pub fn init(alloc: Allocator, io: Io) !Self {
    var http_client = core.HttpClient.init(alloc, io, .{ .retries = 3 });
    errdefer http_client.deinit();

    var spiders = try std.ArrayList(spider.Crawler).initCapacity(alloc, 8);
    errdefer spiders.deinit(alloc);

    return .{
        .alloc = alloc,
        .io = io,
        .http_client = http_client,
        .spiders = spiders,
    };
}

pub fn search(self: *Self, alloc: Allocator, key: []const u8) !domain.Nfo {
    _ = self;
    std.debug.print("search: {s}\n", .{key});
    return domain.Nfo.init(alloc);
}

pub fn deinit(self: *Self) void {
    self.http_client.deinit();
    for (self.spiders.items) |*s| s.deinit();
    self.spiders.deinit(self.alloc);
    self.* = undefined;
}
