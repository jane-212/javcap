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

    var rateLimiter: ?infra.RateLimiter = null;
    if (options.interval_ms > 0) rateLimiter = infra.RateLimiter.init(io, options.interval_ms);

    return .{
        .alloc = alloc,
        .io = io,
        .client = client,
        .rateLimiter = rateLimiter,
        .retry = options.retry,
    };
}

pub fn fetch(self: *Self, alloc: Allocator, options: FetchOptions) !FetchResult {
    while (self.retry >= 0) {
        if (self.rateLimiter) |*rateLimiter| try rateLimiter.acquire();

        const result = self.fetchInner(alloc, options) catch |err| {
            if (self.retry == 0) return err;
            self.retry -= 1;
            continue;
        };
        errdefer result.deinit();

        return result;
    }

    unreachable;
}

pub fn fetchInner(self: *Self, alloc: Allocator, options: FetchOptions) !FetchResult {
    var body: Io.Writer.Allocating = .init(alloc);
    defer body.deinit();

    const response = try self.client.fetch(.{
        .response_writer = &body.writer,
        .location = options.location,
        .method = options.method,
        .payload = options.payload,
        .headers = options.headers,
        .extra_headers = options.extra_headers,
    });

    const ownedBody = try body.toOwnedSlice();
    errdefer alloc.free(ownedBody);

    return .{
        .alloc = alloc,
        .status = response.status,
        .body = ownedBody,
    };
}

pub const FetchOptions = struct {
    location: http.Client.FetchOptions.Location,
    method: ?http.Method = null,
    payload: ?[]const u8 = null,
    headers: http.Client.Request.Headers = .{},
    extra_headers: []const http.Header = &.{},
};

pub const FetchResult = struct {
    alloc: Allocator,
    status: http.Status,
    body: []const u8,

    pub fn deinit(self: *FetchResult) void {
        self.alloc.free(self.body);
    }
};

pub fn deinit(self: *Self) void {
    self.client.deinit();
}
