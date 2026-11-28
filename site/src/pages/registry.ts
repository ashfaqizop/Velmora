// The content pages that are rendered to static HTML at build time
// (scripts/prerender.mjs) and hydrated in the browser (src/page.ts).
// `path` is where the page lives, and the page's own index.html carries its
// title, description and social tags; `jsonld` adds structured data.
import type { Component } from "svelte";
import { BEST_FREE, KODI, KODI_ALTS, PLEX, STREMIO, STREMIO_ALTS } from "../lib/compare";
import { FAQ } from "../lib/faq";
import AccountDoc from "./AccountDoc.svelte";
import AndroidPage from "./AndroidPage.svelte";
import AltKodi from "./compare/AltKodi.svelte";
import AltStremio from "./compare/AltStremio.svelte";
import BestFree from "./compare/BestFree.svelte";
import VsKodi from "./compare/VsKodi.svelte";
import VsPlex from "./compare/VsPlex.svelte";
import VsStremio from "./compare/VsStremio.svelte";
import ContactPage from "./ContactPage.svelte";
import DisclaimerPage from "./DisclaimerPage.svelte";
import DownloadPage from "./DownloadPage.svelte";
import FaqPage from "./FaqPage.svelte";
import FeaturesPage from "./FeaturesPage.svelte";
import InstallDoc from "./InstallDoc.svelte";
import RequirementsDoc from "./RequirementsDoc.svelte";

export interface PageDef {
  path: string;
  title: string;
  description: string;
  component: Component;
  /** Short name for the breadcrumb. */
  crumb: string;
  jsonld?: () => object[];
}

const ORIGIN = "https://getvelmora.dpdns.org";

const breadcrumbs = (path: string, crumb: string) => ({
  "@type": "BreadcrumbList",
  itemListElement: [
    { "@type": "ListItem", position: 1, name: "Home", item: `${ORIGIN}/` },
    { "@type": "ListItem", position: 2, name: crumb, item: `${ORIGIN}${path}` },
  ],
});

function page(def: Omit<PageDef, "jsonld"> & { type?: string; extra?: () => object[] }): PageDef {
  return {
    ...def,
    jsonld: () => [
      { "@type": def.type ?? "WebPage", "@id": `${ORIGIN}${def.path}#page`, url: `${ORIGIN}${def.path}`, name: def.title, description: def.description, isPartOf: { "@id": `${ORIGIN}/#website` } },
      breadcrumbs(def.path, def.crumb),
      ...(def.extra?.() ?? []),
    ],
  };
}

const faqLd = (faq: { q: string; a: string }[]) => () => [
  { "@type": "FAQPage", mainEntity: faq.map((f) => ({ "@type": "Question", name: f.q, acceptedAnswer: { "@type": "Answer", text: f.a } })) },
];

export const PAGES: Record<string, PageDef> = {
  android: page({
    path: "/android/",
    crumb: "Android",
    title: "Velmora for Android: free movies, series, anime and music app (APK)",
    description: "Download Velmora for Android, free and with no ads: movies, series, anime, live TV, sports, Theatre and music on phones and tablets with Android 8 and up.",
    component: AndroidPage,
    extra: () => [
      {
        "@type": "FAQPage",
        mainEntity: [
          { q: "Is Velmora for Android free?", a: "Yes, free and open source, and the Android app has no ads." },
          { q: "Which Android versions does it support?", a: "Android 8 and newer, on phones and tablets." },
          { q: "Where do I get it?", a: "From getvelmora.dpdns.org/android as an APK. It updates itself and isn't on the Play Store." },
          { q: "Does it work with my desktop account?", a: "Yes. It's the same free account: My List and watch progress sync between Windows and Android." },
        ].map((f) => ({ "@type": "Question", name: f.q, acceptedAnswer: { "@type": "Answer", text: f.a } })),
      },
    ],
  }),
  download: page({
    path: "/download/",
    crumb: "Download",
    title: "Download Velmora for Windows",
    description: "Download Velmora, the free open-source app for movies, series and anime, for Windows and Android. One small installer for Windows 10 and 11, with no card and no subscription.",
    component: DownloadPage,
  }),
  features: page({
    path: "/features/",
    crumb: "Features",
    title: "Features: one search, the best link, a real 4K player | Velmora",
    description: "Everything Velmora does: one search across many sources, the best link picked for you, a built-in 4K player, downloads, Theatre, Velmora Social, music and Ani.",
    component: FeaturesPage,
  }),
  faq: page({
    path: "/faq/",
    crumb: "FAQ",
    title: "FAQ: is Velmora free, safe and legal? | Velmora",
    description: "Answers about Velmora: is it free, what is Max, why Windows warns about the installer, whether it hosts content, accounts, privacy, Theatre and Social.",
    component: FaqPage,
    type: "FAQPage",
    extra: () => [
      {
        "@type": "FAQPage",
        mainEntity: FAQ.map((f) => ({ "@type": "Question", name: f.q, acceptedAnswer: { "@type": "Answer", text: f.a } })),
      },
    ],
  }),
  install: page({
    path: "/docs/install/",
    crumb: "Install guide",
    title: "How to install Velmora on Windows and Android | Velmora",
    description: "Step by step: download Velmora, get past SmartScreen on Windows or allow the APK on Android, sign in, and update. With pictures of each screen.",
    component: InstallDoc,
    type: "TechArticle",
  }),
  requirements: page({
    path: "/docs/system-requirements/",
    crumb: "System requirements",
    title: "Velmora system requirements for Windows and Android | Velmora",
    description: "What Velmora needs: Windows 10 or 11 (64-bit), or Android 8 and up on phones and tablets. Space, internet, video and sound, plus a quick check of your device.",
    component: RequirementsDoc,
    type: "TechArticle",
  }),
  account: page({
    path: "/docs/account/",
    crumb: "Create your account",
    title: "Create your free Velmora account | Velmora",
    description: "How to make your free Velmora account with Google, Discord or an emailed code, what syncs between your PC and phone, and how to export or delete it.",
    component: AccountDoc,
    type: "TechArticle",
  }),
  disclaimer: page({
    path: "/disclaimer/",
    crumb: "Disclaimer and DMCA",
    title: "Disclaimer and DMCA policy | Velmora",
    description: "Velmora hosts no video and searches third-party sites. What that means, what you're responsible for, and how to send a copyright notice.",
    component: DisclaimerPage,
  }),
  contact: page({
    path: "/contact/",
    crumb: "Contact",
    title: "Contact Velmora: bugs, ideas, security and contributing",
    description: "Report a bug, suggest an idea, report a security problem privately, send a copyright notice, or help build Velmora. Where to go for each.",
    component: ContactPage,
    type: "ContactPage",
  }),
  vsStremio: page({
    path: "/velmora-vs-stremio/",
    crumb: "Velmora vs Stremio",
    title: "Velmora vs Stremio: which free app is right for you?",
    description: "Velmora and Stremio compared on price, platforms, where the video comes from, open source and extras, with sources and where Stremio is the better pick.",
    component: VsStremio,
    extra: faqLd(STREMIO.faq),
  }),
  vsKodi: page({
    path: "/velmora-vs-kodi/",
    crumb: "Velmora vs Kodi",
    title: "Velmora vs Kodi: a media center or an app that works at once?",
    description: "Velmora and Kodi compared: price, platforms, add-ons, open source and setup, with sources, and when Kodi is the better choice.",
    component: VsKodi,
    extra: faqLd(KODI.faq),
  }),
  vsPlex: page({
    path: "/velmora-vs-plex/",
    crumb: "Velmora vs Plex",
    title: "Velmora vs Plex: a search app or a server for your own library?",
    description: "Velmora and Plex compared: what each is for, price, ads, platforms and source code, with sources, and when Plex is the better pick.",
    component: VsPlex,
    extra: faqLd(PLEX.faq),
  }),
  stremioAlternatives: page({
    path: "/stremio-alternatives/",
    crumb: "Stremio alternatives",
    title: "Stremio alternatives: 7 apps grouped by what you want",
    description: "Alternatives to Stremio, sorted by what you need: apps that find video, servers for your own library, free licensed catalogs and players. With sources.",
    component: AltStremio,
    extra: faqLd(STREMIO_ALTS.faq),
  }),
  kodiAlternatives: page({
    path: "/kodi-alternatives/",
    crumb: "Kodi alternatives",
    title: "Kodi alternatives: 7 apps grouped by what you want",
    description: "Alternatives to Kodi, sorted by what you used it for: your own library, finding video, free catalogs or just playing files. With sources.",
    component: AltKodi,
    extra: faqLd(KODI_ALTS.faq),
  }),
  bestFree: page({
    path: "/best-free-movie-apps-for-windows/",
    crumb: "Best free movie apps for Windows",
    title: "Best free movie apps for Windows: 9 compared",
    description: "Nine free ways to watch movies on a Windows PC, grouped by how they work, with what each is best for and what to watch out for. Velmora included, honestly.",
    component: BestFree,
    extra: faqLd(BEST_FREE.faq),
  }),
};
