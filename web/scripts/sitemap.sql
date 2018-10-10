select '<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://bible.rs/about</loc>
    </url>'
union all
select '<url>
    <loc>https://bible.rs' || replace(path, ' ', '%20') || '</loc>
</url>'
from (
    select '' as path, null, null as path
    union all
    select '/' || b.name as path, b.id, null
    from books b
    union all
    select distinct '/' || b.name || '/' || v.chapter as path, v.book, v.chapter
    from verses_html v
    inner join books b on b.id = v.book
    order by v.book, v.chapter
)
union all
select '</urlset>';
