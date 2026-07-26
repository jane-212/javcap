const std = @import("std");
const Io = std.Io;

const Self = @This();

io: Io,
mutex: Io.Mutex = .init,
next_allowed_ms: i64 = 0,
interval_ms: i64,

pub fn init(io: Io, interval_ms: i64) Self {
    return .{
        .io = io,
        .interval_ms = interval_ms,
    };
}

pub fn acquire(self: *Self) !void {
    try self.mutex.lock(self.io);
    defer self.mutex.unlock(self.io);

    const now = Io.Timestamp.now(self.io, .awake).toMilliseconds();

    if (now < self.next_allowed_ms) {
        const wait_ms = self.next_allowed_ms - now;
        try Io.sleep(self.io, Io.Duration.fromMilliseconds(wait_ms), .awake);
    }

    const acquired_at = Io.Timestamp.now(self.io, .awake).toMilliseconds();
    self.next_allowed_ms = acquired_at + self.interval_ms;
}
