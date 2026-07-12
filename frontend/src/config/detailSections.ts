export type DetailKind = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book" | "paper";

export const detailSectionMatrix: Record<
  DetailKind,
  { propertyList: boolean; groupList: boolean; staffList: boolean; castList: boolean; streaming: boolean }
> = {
  anime: { propertyList: false, groupList: true, staffList: true, castList: true, streaming: true },
  movie: { propertyList: true, groupList: false, staffList: true, castList: true, streaming: true },
  drama: { propertyList: true, groupList: true, staffList: true, castList: true, streaming: true },
  manga: { propertyList: true, groupList: true, staffList: false, castList: false, streaming: false },
  novel: { propertyList: true, groupList: true, staffList: false, castList: false, streaming: false },
  game: { propertyList: true, groupList: false, staffList: false, castList: false, streaming: false },
  academic_book: { propertyList: true, groupList: false, staffList: false, castList: false, streaming: false },
  paper: { propertyList: true, groupList: false, staffList: false, castList: false, streaming: false },
};
