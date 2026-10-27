// hls.js ships no types for its "./light" export; the light build has the same API as the full one.
declare module "hls.js/light" {
  export { default } from "hls.js";
}
