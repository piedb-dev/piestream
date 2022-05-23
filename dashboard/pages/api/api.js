/*
 * Copyright 2022 Singularity Data
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
import config from "../../config";



class Api {

  constructor() {
    this.baseUrl = config.baseUrl.charAt(config.baseUrl.length - 1) === "/"
      ? config.baseUrl.slice(0, config.baseUrl.length)
      : config.baseUrl;
  }

  async get(url) {
    try {
      const res = await fetch(this.baseUrl + url);
      const data = await res.json();
      return data;
    } catch (e) {
      console.error(e);
      throw Error("Failed to fetch " + url);
    }
  }
}


export default new Api();