import { createSignal } from "solid-js";
import { DEFAULT_ARTICLE_SLUG } from "../reference/articles";

const [isOpen, setIsOpen] = createSignal(false);
const [activeSlug, setActiveSlug] = createSignal(DEFAULT_ARTICLE_SLUG);

export function openReference(slug?: string) {
  if (slug) setActiveSlug(slug);
  setIsOpen(true);
}

export function closeReference() {
  setIsOpen(false);
}

export function showReferenceArticle(slug: string) {
  setActiveSlug(slug);
}

export const referenceOpen = isOpen;
export const referenceSlug = activeSlug;
