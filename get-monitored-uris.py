from dbhandler import DBHandler
import re
from spider import Spider


def main():
    db_file = "/home/lobo/PycharmProjects/dface/dface.sqlite"
    monitored_uris_fname = 'monitored-uris.txt'

    db = DBHandler(db_file)
    spider = Spider(10)

    pages = get_monitored_uris(monitored_uris_fname, spider)

    for page in pages:
        page.compute_hashes()
        db.store_page(page)

    spider.close()
    db.close()


def get_monitored_uris(fname, spider):

    pages = []
    with open(fname, "r") as uri_file:
        for line in uri_file:
            uri = line.strip()

            page = spider.get(uri)
            page.defacement = False
            pages.append(page)

    return pages


def match_regex(txt, regex):
    regex = re.search(regex, txt, re.IGNORECASE)
    if regex:
        return regex.group(1)
    return None


# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    main()
