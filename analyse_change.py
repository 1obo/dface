import operator

from dbhandler import DBHandler


def main():
    db_file = "/home/lobo/PycharmProjects/dface/dface.sqlite"

    db = DBHandler(db_file)

    monitored_pages = db.get_latest_monitored_pages()
    defaced_pages = db.get_defaced_pages()

    changed_pages = []
    for page in monitored_pages:
        # Get last 2 hash sets for each monitored page
        # to see if page has changed significantly
        pages = db.get_last_two_versions(page.uri)
        if pages:
            pages[0].compute_hashes()
            pages[1].compute_hashes()
            similarity = pages[0].compare_hashes(pages[1])
            print(f"URI: {pages[0].uri} Similarity: {similarity}/100\n")
            if similarity < 30:
                changed_pages.append(pages[0])

    possible_defacements = {}
    for monitored_page in changed_pages:
        candidates = {}
        for defaced_sample_page in defaced_pages:
            similarity = monitored_page.compare_hashes(defaced_sample_page)
            if similarity > 30:
                candidates[defaced_sample_page] = similarity
        possible_defacements[monitored_page] = candidates

    if len(possible_defacements) > 0:
        for mon_page, pd in possible_defacements.items():
            for (defacement, score) in sorted(pd.items(), key=operator.itemgetter(1)):
                print(f"Defacement Alert: URI: {mon_page.uri} Similar defacement: {defacement.uri} Confidence: {score}")
                # print(f"URI: {monitored_page.uri} Defacement: {defaced_sample_page.uri} Similarity: {similarity}/100")
    else:
        print("No defacements detected")

    db.close()


if __name__ == '__main__':
    main()
