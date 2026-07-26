const std = @import("std");
const http = std.http;
const Allocator = std.mem.Allocator;
const Io = std.Io;
const infra = @import("root.zig");

const Self = @This();

alloc: Allocator,
io: Io,
client: http.Client,
rateLimiter: ?infra.RateLimiter,
retry: u32,

pub const Options = struct {
    interval_ms: i64 = 0,
    retry: u32 = 0,
};

pub fn init(
    alloc: Allocator,
    io: Io,
    options: Options,
) Self {
    const client: http.Client = .{
        .allocator = alloc,
        .io = io,
    };
    errdefer client.deinit();

    const rateLimiter = null;
    if (options.interval_ms > 0) rateLimiter = infra.RateLimiter.init(io, options.interval_ms);

    return .{
        .alloc = alloc,
        .io = io,
        .client = client,
        .rateLimiter = rateLimiter,
        .retry = options.retry,
    };
}

pub fn fetch(self: *Self, options: http.Client.FetchOptions) !http.Client.FetchResult {
    while (self.retry >= 0) {
        if (self.rateLimiter) |rateLimiter| try rateLimiter.acquire();

        const result = self.client.fetch(options) catch |err| {
            if (self.retry == 0) return err;
            self.retry -= 1;
            continue;
        };

        return result;
    }

    unreachable;
}

pub fn deinit(self: *Self) void {
    self.client.deinit();
    self.* = undefined;
}
