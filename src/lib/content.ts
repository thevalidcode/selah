/**
 * Plain-language names for stored content types.
 *
 * The database stores `scripture`, `media`, `lyrics`… which mean nothing to
 * someone running a service. Every screen that shows a content type goes
 * through here so the wording can only be defined once.
 */
export function friendlyContentType(type: string): string {
  switch (type) {
    case "scripture":
      return "Bible verse";
    case "lyrics":
      return "song words";
    case "song":
      return "song";
    case "announcement":
      return "notice";
    case "slide":
      return "slide";
    case "media":
    case "image":
      return "picture";
    case "video":
      return "video";
    case "text":
      return "words";
    default:
      return type;
  }
}
