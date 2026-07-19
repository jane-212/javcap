const std = @import("std");
const Allocator = std.mem.Allocator;

const Self = @This();

pub const Actor = struct {
    name: ?[]const u8 = null,
    role: ?[]const u8 = null,
    order: u32 = 0,
    thumb: ?[]const u8 = null,
    type: ?[]const u8 = null,

    pub fn deinit(self: *Actor, alloc: Allocator) void {
        if (self.name) |v| alloc.free(v);
        if (self.role) |v| alloc.free(v);
        if (self.thumb) |v| alloc.free(v);
        if (self.type) |v| alloc.free(v);
        self.* = undefined;
    }
};

pub const Rating = struct {
    name: ?[]const u8 = null,
    value: ?[]const u8 = null,
    votes: ?[]const u8 = null,
    max: ?[]const u8 = null,

    pub fn deinit(self: *Rating, alloc: Allocator) void {
        if (self.name) |v| alloc.free(v);
        if (self.value) |v| alloc.free(v);
        if (self.votes) |v| alloc.free(v);
        if (self.max) |v| alloc.free(v);
        self.* = undefined;
    }
};

pub const UniqueId = struct {
    type: ?[]const u8 = null,
    value: ?[]const u8 = null,
    default: bool = false,

    pub fn deinit(self: *UniqueId, alloc: Allocator) void {
        if (self.type) |v| alloc.free(v);
        if (self.value) |v| alloc.free(v);
        self.* = undefined;
    }
};

pub const Art = struct {
    poster: ?[]const u8 = null,
    fanart: ?[]const u8 = null,
    banner: ?[]const u8 = null,
    thumb: ?[]const u8 = null,
    logo: ?[]const u8 = null,
    clearart: ?[]const u8 = null,
    clearlogo: ?[]const u8 = null,
    discart: ?[]const u8 = null,
    landscape: ?[]const u8 = null,

    pub fn deinit(self: *Art, alloc: Allocator) void {
        if (self.poster) |v| alloc.free(v);
        if (self.fanart) |v| alloc.free(v);
        if (self.banner) |v| alloc.free(v);
        if (self.thumb) |v| alloc.free(v);
        if (self.logo) |v| alloc.free(v);
        if (self.clearart) |v| alloc.free(v);
        if (self.clearlogo) |v| alloc.free(v);
        if (self.discart) |v| alloc.free(v);
        if (self.landscape) |v| alloc.free(v);
        self.* = undefined;
    }
};

alloc: Allocator,

title: ?[]const u8 = null,
originaltitle: ?[]const u8 = null,
sortname: ?[]const u8 = null,
sorttitle: ?[]const u8 = null,
plot: ?[]const u8 = null,
tagline: ?[]const u8 = null,
outline: ?[]const u8 = null,

genres: ?[][]const u8 = null,
tags: ?[][]const u8 = null,
styles: ?[][]const u8 = null,
studios: ?[][]const u8 = null,
directors: ?[][]const u8 = null,
writers: ?[][]const u8 = null,
countries: ?[][]const u8 = null,

actors: ?[]Actor = null,

year: ?u32 = null,
runtime: ?u32 = null,

rating: ?[]const u8 = null,
ratings: ?[]Rating = null,
mpaa: ?[]const u8 = null,
customrating: ?[]const u8 = null,

premiered: ?[]const u8 = null,
aired: ?[]const u8 = null,
releasedate: ?[]const u8 = null,
enddate: ?[]const u8 = null,
dateadded: ?[]const u8 = null,
lastplayed: ?[]const u8 = null,

trailer: ?[]const u8 = null,
aspectratio: ?[]const u8 = null,
language: ?[]const u8 = null,
countrycode: ?[]const u8 = null,

thumb: ?[]const u8 = null,
fanart: ?[]const u8 = null,

art: Art = .{},

lockdata: ?bool = null,
lockedfields: ?[]const u8 = null,
displayorder: ?[]const u8 = null,
watched: ?bool = null,
playcount: ?u32 = null,

fileinfo: ?[]const u8 = null,

imdbid: ?[]const u8 = null,
imdb_id: ?[]const u8 = null,
tvdbid: ?[]const u8 = null,
tmdbid: ?[]const u8 = null,
tvrageid: ?[]const u8 = null,
zap2itid: ?[]const u8 = null,
audiodbartistid: ?[]const u8 = null,
audiodbalbumid: ?[]const u8 = null,
musicbrainzartistid: ?[]const u8 = null,
musicbrainzalbumartistid: ?[]const u8 = null,
musicbrainzalbumid: ?[]const u8 = null,
musicbrainzreleasegroupid: ?[]const u8 = null,

uniqueids: ?[]UniqueId = null,

showtitle: ?[]const u8 = null,
season: ?u32 = null,
episode: ?u32 = null,
seasonnumber: ?u32 = null,
displayepisode: ?u32 = null,
displayseason: ?u32 = null,
airsafter_season: ?u32 = null,
airsbefore_episode: ?u32 = null,
airsbefore_season: ?u32 = null,

set: ?[]const u8 = null,
collectionnumber: ?u32 = null,
formed: ?[]const u8 = null,
disbanded: ?[]const u8 = null,

pub fn init(alloc: Allocator) Self {
    return .{ .alloc = alloc };
}

pub fn deinit(self: *Self) void {
    inline for (.{
        "title",                     "originaltitle",            "sortname",
        "sorttitle",                 "plot",                     "tagline",
        "outline",                   "rating",                   "mpaa",
        "customrating",              "premiered",                "aired",
        "releasedate",               "enddate",                  "dateadded",
        "lastplayed",                "trailer",                  "aspectratio",
        "language",                  "countrycode",              "thumb",
        "fanart",                    "lockedfields",             "displayorder",
        "fileinfo",                  "imdbid",                   "imdb_id",
        "tvdbid",                    "tmdbid",                   "tvrageid",
        "zap2itid",                  "audiodbartistid",          "audiodbalbumid",
        "musicbrainzartistid",       "musicbrainzalbumartistid", "musicbrainzalbumid",
        "musicbrainzreleasegroupid", "showtitle",                "set",
        "formed",                    "disbanded",
    }) |field_name| {
        if (@field(self, field_name)) |v| self.alloc.free(v);
    }

    inline for (.{
        "genres",    "tags",    "styles",    "studios",
        "directors", "writers", "countries",
    }) |field_name| {
        if (@field(self, field_name)) |slice| {
            for (slice) |item| self.alloc.free(item);
            self.alloc.free(slice);
        }
    }

    if (self.actors) |actors| {
        for (actors) |*a| a.deinit(self.alloc);
        self.alloc.free(actors);
    }

    if (self.ratings) |ratings| {
        for (ratings) |*r| r.deinit(self.alloc);
        self.alloc.free(ratings);
    }

    if (self.uniqueids) |uniqueids| {
        for (uniqueids) |*u| u.deinit(self.alloc);
        self.alloc.free(uniqueids);
    }

    self.art.deinit(self.alloc);

    self.* = undefined;
}
