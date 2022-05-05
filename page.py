import io

import ssdeep
import imagehash
from PIL import Image
from scipy.spatial.distance import hamming

class Page:

    def __init__(self, uri):
        self.labels = {}
        self.uri = uri
        self.type = None
        self.image = None
        self.html = None
        self.ts = None

    def put(self, key, value):
        self.labels[key] = value

    def get(self, key):
        if key in self.labels:
            return self.labels.get(key)
        return None

    def get_keys(self):
        return self.labels.keys()

    def compute_hashes(self):
        ssdeep_hash = ssdeep.hash(self.html)
        self.labels['ssdeep_hash'] = ssdeep_hash

        stream = io.BytesIO(self.image)
        image = Image.open(stream)

        phash = str(imagehash.phash(image, 16, 4))
        self.labels['p_hash'] = phash

        dhash = str(imagehash.dhash(image, 16))
        self.labels['d_hash'] = dhash

    def compare_hashes(self, other_page):
        # Compare ssdeep values
        ssdeep_value = ssdeep.compare(self.get('ssdeep_hash'), other_page.get('ssdeep_hash'))

        # Compare d_hash values
        d_value = 100 - (hamming(self.get('d_hash'), other_page.get('d_hash')) * len(self.get('p_hash')))

        # Compare p_hash values
        p_value = 100 - (hamming(self.get('p_hash'), other_page.get('p_hash')) * len(self.get('p_hash')))
        score = int((ssdeep_value + p_value + d_value) / 3)

        print(f"URI1: {self.uri} (TS: {self.ts})   URI2: {other_page.uri} (TS: {other_page.ts})")
        print(f"ssdeep_hash1: {self.get('ssdeep_hash')}\n"
              f"ssdeep_hash2: {other_page.get('ssdeep_hash')}\n")
        print(f"p_hash1: {self.get('p_hash')}\n"
              f"p_hash2: {other_page.get('p_hash')}\n")
        print(f"d_hash1: {self.get('d_hash')}\n"
              f"d_hash2: {other_page.get('d_hash')}\n\n"
              f"Score: {score}/100\n")

        return score




    def __str__(self):
        return (f"URI: {self.uri}   TS: {self.ts}   Type:{self.type}\n"
                f"HTML: {self.html[:20]}\n"
                f"IMG: {self.image[:20]}\n"
                f"LABELS: {self.labels}")

