import type { Api, Episode, Info, ListKind, SearchHit, SearchPage, Stream } from "./api";

// A factory so the fake data is not top-level side effects: Rollup can then drop it from production builds.
export function makeDemoApi(): Api {
  const T = (id: number, title: string, rating: number, genres = ["Action", "Adventure", "Comedy"]): SearchHit =>
    ({ id, title, genres, kind: "Animeserie/TV", rating });
  const HITS: SearchHit[] = [
    T(53, "One Piece", 9.64), T(59782, "Tensei shitara Ken deshita 2", 7.41, ["Action", "Fantasy"]), T(14234, "Boruto: Naruto Next Generations", 7.1),
    T(2162, "Naruto Shippuuden", 8.5), T(9788, "Dragon Ball Super", 8.2), T(1874, "Steins;Gate", 9.1, ["Drama", "SciFi"]),
    T(45, "Naruto", 8.37), T(61314, "Another Title", 7.0), T(3417, "Bleach", 8.0), T(3478, "Fairy Tail", 7.9),
  ];
  const rotate = (n: number) => [...HITS.slice(n), ...HITS.slice(0, n)];
  const lists: Record<ListKind, SearchHit[]> = { airing: rotate(0), rating: rotate(3), clicks: [] };

  const epis = (n: number): Episode[] => Array.from({ length: n }, (_, i) => ({ number: i + 1, langs: i % 7 === 6 ? ["engsub"] : ["engsub", "engdub", "gersub", "gerdub"] }));
  const stream = (type: string, name: string): Stream => ({ code: "demo", type, name, replace: "https://stream.proxer.me/embed-#-728x504.html" });

  return {
    login: async () => true,
    autoLogin: async () => true,
    logout: async () => {},
    search: async (query: string): Promise<SearchPage> => ({ hits: HITS.filter((h) => h.title.toLowerCase().includes(query.toLowerCase()) || !query), has_next: false }),
    getList: async (kind: ListKind): Promise<SearchPage> => ({ hits: lists[kind], has_next: false }),
    info: async (id: number): Promise<Info> => {
      const h = HITS.find((x) => x.id === id) ?? HITS[0];
      return { title: h.title, fields: { Status: id === 53 || id === 59782 ? "Airing" : "Abgeschlossen", Genre: h.genres.join(", "), Season: "Herbst 2002" } };
    },
    episodes: async (_id: number, page = 1) => ({
      episodes: epis(50).map((e) => ({ ...e, number: e.number + (page - 1) * 50 })),
      pages: [1, 2, 3, 4].map((p) => ({ page: p, label: `${(p - 1) * 50 + 1} - ${p * 50}` })),
    }),
    streams: async () => [stream("proxer-stream", "Proxer-Stream"), stream("ps-test", "Proxer-Stream Airing")],
    resolve: async () => { throw "Network"; },
  };
}
