// The content pages that are rendered to static HTML at build time
// (scripts/prerender.mjs) and hydrated in the browser (src/page.ts).
// `path` is where the page lives, and the page's own index.html carries its
// title, description and social tags; `jsonld` adds structured data.
import type { Component } from "svelte";
import { FAQ } from "../lib/faq";
import AccountDoc from "./AccountDoc.svelte";
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

export const PAGES: Record<string, PageDef> = {
  download: page({
    path: "/download/",
    crumb: "Download",
    title: "Download Velmora for Windows",
    description: "Download Velmora, the free open-source Windows app for movies, series and anime. One small installer for Windows 10 and 11, with no card and no subscription.",
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
    title: "How to install Velmora on Windows | Velmora",
    description: "Step by step: download Velmora, get past the Windows SmartScreen warning, open it for the first time, sign in, and update. With pictures of each screen.",
    component: InstallDoc,
    type: "TechArticle",
  }),
  requirements: page({
    path: "/docs/system-requirements/",
    crumb: "System requirements",
    title: "Velmora system requirements for Windows | Velmora",
    description: "What Velmora needs: Windows 10 or 11 (64-bit), no administrator rights, disk space, internet, and what makes 4K smooth. Plus a quick check of your own PC.",
    component: RequirementsDoc,
    type: "TechArticle",
  }),
  account: page({
    path: "/docs/account/",
    crumb: "Create your account",
    title: "Create your free Velmora account | Velmora",
    description: "How to make your free Velmora account with Google, Discord or an emailed code, what your profile holds, what syncs, and how to export or delete it.",
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
};
