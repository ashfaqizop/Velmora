// Adsterra units for the website. Ad codes are public (they're in every page
// that shows them), so they live here instead of in environment variables.
// The app's own ad pages are in public/ad/.

export const BANNER_HOST = "https://www.highrevenueformat.com";

export const BANNERS = {
  "728x90": "a324f1c44f0f470fe3c3f5048c8ebb2a",
  "468x60": "5ba9c21f1555815dbdf7cfee39a7d0e6",
  "320x50": "a16f850c043bf8a8f588f1a6ff394ef8",
  "300x250": "a6dec928ac25348d2c54d4f228842971",
  "160x600": "3b8bbe4860d7612e6caddcc1ede88026",
  "160x300": "58f5983b71b409f96dcb2fa2238642f7",
} as const;

export type BannerSize = keyof typeof BANNERS;

export const NATIVE = {
  script: "https://pl32247109.profitableratecpmnetwork.com/e3cc54a43bbcfedbc1b5641793853c5a/invoke.js",
  container: "container-e3cc54a43bbcfedbc1b5641793853c5a",
};

export const SOCIAL_BAR = "https://pl32247110.profitableratecpmnetwork.com/2f/f4/fe/2ff4fe628d6ac7ec9253c48534fab7d9.js";
export const POPUNDER = "https://pl32247108.profitableratecpmnetwork.com/0a/e0/1f/0ae01faf5635225d3fee0cc31013b990.js";
export const SMARTLINK = "https://www.profitableratecpmnetwork.com/mtpjrjcfv?key=0249b357f896b6490f4dd1a8115f3bff";
