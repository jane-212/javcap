const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const http = std.http;

const Self = @This();

alloc: Allocator,
io: Io,
client: http.Client,
retries: i32,

pub const InitOptions = struct {
    retries: i32 = 0,
};

pub fn init(alloc: Allocator, io: Io, options: InitOptions) Self {
    const client: http.Client = .{
        .allocator = alloc,
        .io = io,
    };
    errdefer client.deinit();

    return .{
        .alloc = alloc,
        .io = io,
        .client = client,
        .retries = options.retries,
    };
}

pub fn fetch(self: *Self, alloc: Allocator, options: http.Client.FetchOptions) ![]u8 {
    var retries = self.retries;
    if (retries <= 0) return try self.fetchInner(alloc, options);
    while (retries >= 0) {
        retries -= 1;
        const response = self.fetchInner(alloc, options) catch |err| switch (err) {
            error.StatusNotOk,
            error.ConnectionRefused,
            error.ConnectionResetByPeer,
            error.HostUnreachable,
            error.NetworkUnreachable,
            error.Timeout,
            error.ConnectionPending,
            error.WouldBlock,
            error.NetworkDown,
            error.SystemResources,
            error.NameServerFailure,
            error.NoAddressReturned,
            error.DetectingNetworkConfigurationFailed,
            error.ReadFailed,
            error.HttpRequestTruncated,
            error.HttpConnectionClosing,
            error.HttpChunkInvalid,
            error.HttpChunkTruncated,
            error.WriteFailed,
            => continue,
            else => return err,
        };
        return response;
    }
    return error.MaxRetriesReached;
}

fn fetchInner(self: *Self, alloc: Allocator, options: http.Client.FetchOptions) ![]u8 {
    var response_writer = std.Io.Writer.Allocating.init(alloc);
    defer response_writer.deinit();
    const writer = &response_writer.writer;
    options.response_writer = writer;
    const result = try self.client.fetch(options);
    if (result.status != .ok) return error.StatusNotOk;
    const response = try response_writer.toOwnedSlice();
    errdefer alloc.free(response);
    return response;
}

pub fn deinit(self: *Self) void {
    self.client.deinit();
    self.* = undefined;
}
