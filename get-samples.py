from dbhandler import DBHandler
import re
from spider import Spider
from selenium.webdriver.common.by import By


def main():
    db_file = "/home/lobo/PycharmProjects/dface/dface.sqlite"
    start_page = 1
    max_samples = 25
    mirrored_pages = []  # "http://zone-h.org/mirror/id/39688641"]

    db = DBHandler(db_file)
    spider = Spider(0.125)

    pages = []
    if len(mirrored_pages) > 0:
        for page in mirrored_pages:
            pages.append(get_mirrored_page(page, spider))
    else:
        pages = get_zoneh_samples(start_page, max_samples, spider)

    for page in pages:
        page.compute_hashes()
        db.store_page(page)

    spider.close()
    db.close()


def get_zoneh_samples(start_page, max_samples, spider):
    sample_url = "http://zone-h.org/archive/special=1/page="
    page = start_page

    links = []
    while len(links) < max_samples:
        sample_page = sample_url + str(page)
        spider.get(sample_page)
        new_links = spider.find_elements(by=By.LINK_TEXT, value="mirror")
        if len(new_links) == 0:
            print("Could not find more links")
            break
        for link in new_links:
            links.append(link.get_attribute("href"))
            if len(links) >= max_samples:
                break
        page = page + 1

    pages = []
    for link in links:
        print(f"Progress: {len(pages)} of {len(links)}")
        page = get_mirrored_page(link, spider)
        pages.append(page)
    return pages


def get_mirrored_page(link, spider):
    spider.get(link)
    domain = match_regex(spider.find_element(
        by=By.XPATH,
        value="(//li[@class='defaces'])[1]").get_attribute("innerHTML"),
                         '.+?<\/strong>\s*(\S+?)$')
    ip = match_regex(spider.find_element(
        by=By.XPATH,
        value="(//li[@class='defacet'])[1]").get_attribute("innerHTML"),
                     '.+?<\/strong>\s*(\S+)')
    system = match_regex(spider.find_element(
        by=By.XPATH,
        value="(//li[@class='defacef'])[2]").get_attribute("innerHTML"),
                         '.+?<\/strong>\s*(.+?)$')
    webserver = match_regex(spider.find_element(
        by=By.XPATH,
        value="(//li[@class='defaces'])[2]").get_attribute("innerHTML"),
                            '.+?<\/strong>\s*(.+?)$')

    sample = spider.find_element(by=By.XPATH, value="//iframe").get_attribute("src")

    page = spider.get(sample)
    page.defacement = True
    page.put('domain', domain)
    page.put('ip', ip)
    page.put('system', system)
    page.put('webserver', webserver)

    return page


def match_regex(txt, regex):
    regex = re.search(regex, txt, re.IGNORECASE)
    if regex:
        return regex.group(1)
    return None


# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    main()
