// The home page: the write-up in index.html gets the title band and the footer, and links from
// before it existed are sent on to the page they meant.

import { band, footer, navLinks, siteCounts } from './chrome';
import { forwardFor } from './forward';
import { galleryEntries } from './gallery';
import { fixtures } from './levels';

function page(app: HTMLElement): void {
  const onward = forwardFor(location.search);
  if (onward !== undefined) {
    location.replace(onward);
    return;
  }
  const counts = siteCounts(galleryEntries(), fixtures());
  const nav = navLinks([
    ['arena', 'arena.html'],
    ['gallery', 'gallery.html'],
    ['leaderboard', 'leaderboard.html'],
    ['level', 'level.html'],
  ]);
  app.prepend(band('Clattertrap', nav, counts));
  app.append(footer(counts));
}

const app = document.getElementById('app');
if (app) {
  page(app);
}
