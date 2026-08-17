export type DetailKind = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book" | "paper";

export const detailSectionMatrix: Record<
  DetailKind,
  { propertyList: boolean; groupList: boolean; staffList: boolean; castList: boolean; themeSongs: boolean; streaming: boolean; images: boolean }
> = {
  anime: { propertyList: false, groupList: true, staffList: true, castList: true, themeSongs: true, streaming: true, images: true },
  movie: { propertyList: true, groupList: false, staffList: true, castList: true, themeSongs: true, streaming: true, images: true },
  drama: { propertyList: true, groupList: true, staffList: true, castList: true, themeSongs: true, streaming: true, images: true },
  manga: { propertyList: true, groupList: true, staffList: false, castList: false, themeSongs: false, streaming: false, images: true },
  novel: { propertyList: true, groupList: true, staffList: false, castList: false, themeSongs: false, streaming: false, images: true },
  game: { propertyList: true, groupList: false, staffList: false, castList: false, themeSongs: false, streaming: false, images: true },
  academic_book: { propertyList: true, groupList: false, staffList: false, castList: false, themeSongs: false, streaming: false, images: true },
  paper: { propertyList: true, groupList: false, staffList: false, castList: false, themeSongs: false, streaming: false, images: true },
};
