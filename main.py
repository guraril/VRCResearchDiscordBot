# stable版だけがいいとかがあれば言って
# NOTE: Update、ログに書き出す？要らない？
# NOTE: GitHub API、ログインせずに使っているので60req/hの制限がある

import urllib.request
import json
import time
import schedule

def getUpdatesJob():
    request_repo_list = ["https://api.github.com/repos/anatawa12/AvatarOptimizer/releases/latest",
                        "https://api.github.com/repos/bdunderscore/modular-avatar/releases/latest",
                        "https://api.github.com/repos/lilxyzw/liltoon/releases/latest",
                        "https://api.github.com/repos/ReinaS-64892/TexTransTool/releases/latest",
                        "https://api.github.com/repos/vrchat/packages/releases/latest",
                        "https://api.github.com/repos/VRCFury/VRCFury/releases/latest",
                        "https://api.github.com/repos/lilxyzw/lilycalInventory/releases/latest",]

    cache_file = open("./cache.json", "r")
    cache_json = json.load(cache_file)
    cache_file.close()
    cache_file = open("./cache.json", "w")
    write_obj = {
        "releases": [
            "",
            "",
            "",
            "",
            "",
            "",
            ""
        ]
    }

    def getJsonList():
        json_list = []
        for i in request_repo_list:
            response = urllib.request.urlopen(i)
            json_list.append(response.read())
        return json_list

    json_list = getJsonList()
    for i in range(len(json_list)):
        release_url = json.loads(json_list[i])["html_url"]
        write_obj["releases"][i] = release_url
        if write_obj["releases"][i] != cache_json["releases"][i]:
            print("TODO: Post to discord")
            match i:
                case 0:
                    print("AAO: Avatar Optimizer has a new version!")
                case 1:
                    print("Modular Avatar has a new version!")
                case 2:
                    print("lilToon has a new version!")
                case 3:
                    print("TexTransTool has a new version!")
                case 4:
                    print("VRCSDK has a new version!")
                case 5:
                    print("VRCFury has a new version!")
                case 6:
                    print("lilycalInventory has a new version!")

    json.dump(write_obj, cache_file)
    cache_file.close()

if __name__ == "__main__":
    schedule.every(2).hours.do(getUpdatesJob)
    while True:
        schedule.run_pending()
        time.sleep(1800)
