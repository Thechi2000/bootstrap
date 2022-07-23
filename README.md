<h1>Bootstrap</h1>
<h2>Overview</h2>
<p>
    The goal of this program is to update an application using a remote server, then start it, to ensure that the application will always be up to date.
    It uses compile time environment variables thus allowing easy reconfiguration without the need of an extra file at runtime.<br> 
    To achieve that, it goes through the following steps:
</p>
<h2>Behavior</h2>
<p>
The bootstrap first send a GET request to the server, at the url specified by <strong>FETCH_URL</strong>. <br>
It expects to receive a json formatted as follows:

```
    {
        base_url: String - Url of the root of the application
        algorithm: String - Name of the algorithm to checksum the files
        [{
            path: String - Path of the file (relative to the root of the application
            hash: String - Checksum of the file (Crockford Base32 Encoding)
        }] - Array containing the informations for each file of the application
    }
```
<p>
Available hash algorithm are SHA256, SHA384, SHA512, SHA512_256
</p>
</p>
<p>
Then, it computes the checksum of all files in the working directory and compare it to the one received, and updates the file if it differs. If the file is not referenced in the json, it is deleted.
The downloaded file url are computed by merging <strong>base_url</strong> and <strong>path</strong>.
</p>
<p>
When the update is completed, it starts the executable designated by <strong>EXECUTABLE_NAME</strong> 
</p>
<h2>Compilation</h2>
<p>
All used libraries are platform independent, thus making the application easy to cross compile.

The following environment are needed at compile time:
</p>
<ul>
<li><strong>FETCH_URL</strong>: Url where to fetch the json</li>
<li><strong>EXECUTABLE_NAME</strong>: Name of the executable of the application, started after update</li>
</ul>
