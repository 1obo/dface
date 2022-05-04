from pyvirtualdisplay import Display
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.common.alert import Alert
from selenium.webdriver.common.keys import Keys

# from selenium_chrome import Chrome
# from selenium_firefox import Firefox

import time
from Page import Page


class Spider:

    def __init__(self, rate_limit=2):
        self.rate = 1/rate_limit
        self.settle_delay = 1
        self.last_request_ts = None

        # self.display = Display(visible=0, size=(1920, 1080))
        # self.display.start()

        self.browser = webdriver.Firefox()
        # self.browser = webdriver.Chrome()

    def get(self, url):
        self.rate_limit()
        print(f"Getting: {url}")
        self.browser.set_window_size(1024, 1080)
        self.browser.get(url)
        time.sleep(self.settle_delay) # Sleep for 1 sec to let hte page do its thing
        page = Page(url)
        page.ts = time.time()
        # page.image = self.browser.get_screenshot_as_png()
        page.image = self.browser.get_full_page_screenshot_as_png()
        page.html = self.browser.page_source
        return page

    def find_element(self, by, value):
        return self.browser.find_element(by=by, value=value)

    def find_elements(self, by, value):
        return self.browser.find_elements(by=by, value=value)

    def rate_limit(self):
        current_ts = time.time()

        if self.last_request_ts:
            delta = current_ts - self.last_request_ts
            wait = self.rate - delta

            if wait > 0:
                print(f"Sleeping for: {wait}")
                time.sleep(wait)

        self.last_request_ts = time.time()

    def close(self):
        self.browser.close()
