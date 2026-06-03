namespace UpdateNuspecTool;

public static class NuspecProcessorHelper
{
    public static void Process(string file, string path)
    {
        try
        {
            // Инициализация коллекций для результатов
            var updatedReferences = new List<Dependency>();
            var addedReferences = new List<Dependency>();
            var noChangesReferences = new List<Dependency>();
            var deletedReferences = new List<Dependency>();
            var outdatedReferences = new Dictionary<string, string>();

            var stopwatch = new Stopwatch();
            stopwatch.Start();

            // Загрузка nuspec xml файла
            var nuspecData = XDocument.Load(file);
            XNamespace nuspecDocNamespace = nuspecData.Root!.Name.Namespace;

            // Получение имени проекта из nuspec
            var projectName = nuspecData.Descendants(nuspecDocNamespace + "metadata")
                .Select(x => x.Element(nuspecDocNamespace + "id"))
                .First()
                ?.Value;

            // Получение списка Dependency из nuspec
            var dependencies = nuspecData.Descendants(nuspecDocNamespace + "dependencies")
                .SelectMany(x => x.Elements())
                .Select(p => new Dependency(
                    Name: p.Attribute("id")!.Value,
                    Version: p.Attribute("version")!.Value))
                .ToList();

            if (projectName == null)
            {
                ConsoleHelper.WriteLine($"ProjectName not found in: {file}", ConsoleColor.Red);
                return;
            }

            // Формируем путь к файлу проекта по информации из nuspec
            var projectFilePath = Path.Combine(path, projectName + ".csproj");
            if (!File.Exists(projectFilePath))
            {
                ConsoleHelper.WriteLine($"ProjectFile: {projectName} not found in: {path}", ConsoleColor.Red);
                return;
            }

            ConsoleHelper.Write($"Processing project: ", ConsoleColor.Gray);
            ConsoleHelper.Write($"{projectFilePath} \n", ConsoleColor.Cyan);


            // Загрузка xml файла проекта
            var projectData = XDocument.Load(projectFilePath);

            // Получение packageReferences из файла проекта
            var packageReferences = projectData.Descendants("ItemGroup")
                .SelectMany(p => p.Elements("PackageReference"))
                .Select(x => new Dependency(Name: x.Attribute("Include")!.Value, Version:x.Attribute("Version")!.Value))
                .ToList();

            foreach (var item in packageReferences)
            {
                // Поиск dependency с таким же именем как в файле проекте
                var dependencyToUpdate = dependencies
                    .Where(p => p.Name == item.Name)
                    .ToList();

                // Если dependency не найден, добавляем его в список "Новых"
                if (!dependencyToUpdate.Any())
                {
                    addedReferences.Add(new (item.Name, item.Version));
                    continue;
                }

                // Для всех найденных dependency сравниваем версию - заносим dependency либо в "Обновленные", либо в "Без Изменений"
                foreach (var dependency in dependencyToUpdate)
                {
                    if (dependency.Version != item.Version)
                    {
                        outdatedReferences.Add(dependency.Name, dependency.Version);
                        updatedReferences.Add(new (item.Name, item.Version));
                        continue;
                    }

                    noChangesReferences.Add(new (item.Name, item.Version));
                }
            }

            // Формируем коллекцию для сортировки
            var orderedDependencyList = new List<Dependency>();
            orderedDependencyList.AddRange(updatedReferences);
            orderedDependencyList.AddRange(noChangesReferences);
            orderedDependencyList.AddRange(addedReferences);

            // Формируем коллекцию всех dependency с "Cross." в названии
            var crossList = orderedDependencyList.Where(p => p.Name.StartsWith("Cross.")).OrderBy(p => p.Name).ToList();
            orderedDependencyList.RemoveAll(p => crossList.Contains(p));

            // Формируем коллекцию всех dependency c "Boilerplate" в названии
            var boilerplateList = orderedDependencyList.Where(p => p.Name.Contains("Boilerplate")).OrderBy(p => p.Name).ToList();
            orderedDependencyList.RemoveAll(p => boilerplateList.Contains(p));

            // Формируем коллекцию всех dependency с ".Api.Contract" в названии
            var apiContractList = orderedDependencyList.Where(p => p.Name.Contains(".Api.Contract")).OrderBy(p => p.Name).ToList();
            orderedDependencyList.RemoveAll(p => apiContractList.Contains(p));

            // Упорядочиваем коллекцию по алфавиту
            orderedDependencyList = orderedDependencyList.OrderBy(p => p.Name).ToList();

            // Формируем финальную коллекцию которая добавится в nuspec
            var resultList = new List<Dependency>();
            resultList.AddRange(crossList);
            resultList.AddRange(boilerplateList);
            resultList.AddRange(apiContractList);
            resultList.AddRange(orderedDependencyList);

            // Формируем список удаленных
            var dependencyNames = dependencies.Select(p => p.Name).ToList();
            var resultNames = resultList.Select(p => p.Name).ToList();
            var deletedNames = dependencyNames.Where(p => !resultNames.Contains(p)).ToList();
            deletedReferences = dependencies.Where(p => deletedNames.Contains(p.Name)).ToList();

            // Удаляем все dependencies из nuspec
            nuspecData.Descendants(nuspecDocNamespace + "metadata")
                .Select(p => p.Element(nuspecDocNamespace + "dependencies"))
                .First()
                !.RemoveAll();

            // Заполняем nuspec правильными значениями
            foreach (var value in resultList)
            {
                nuspecData.Descendants(nuspecDocNamespace + "dependencies")
                    .First()
                    .Add(new XElement(nuspecDocNamespace + "dependency", new XAttribute("id", value.Name), new XAttribute("version", value.Version)));
            }

            // Сохранение references в файл
            nuspecData.Save(file);

            // Функция для вывода результатов в консоль
            ConsoleHelper.ShowResult(
                updatedReferences: updatedReferences,
                noChangesReferences: noChangesReferences,
                addedReferences: addedReferences,
                deletedReferences: deletedReferences,
                outdatedReferences: outdatedReferences);

            stopwatch.Stop();

            TimeSpan ts = stopwatch.Elapsed;
            string elapsedTime = String.Format("{0:00}:{1:00}:{2:00}.{3:00}",
                ts.Hours, ts.Minutes, ts.Seconds,
                ts.Milliseconds);

            Console.WriteLine($"Elapsed : {elapsedTime}");
        }
        catch (Exception ex)
        {
            throw;
        }
    }
}
